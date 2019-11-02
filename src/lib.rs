use std::path::PathBuf;

mod cli;
mod dependencies;
mod error;
#[cfg(unix)]
mod execv;
mod lock;
mod operations;
mod paths;
mod python_info;
mod run;
mod settings;
mod ui;
#[cfg(windows)]
mod win_job;

use crate::cli::commands;
pub use crate::cli::syntax::Command;
use crate::cli::syntax::SubCommand;
pub use crate::error::*;
use crate::lock::BumpType;
use crate::operations::UpdateOptions;
use crate::paths::{Paths, PathsResolver};
pub use crate::paths::{DEV_LOCK_FILENAME, PROD_LOCK_FILENAME};
use crate::python_info::PythonInfo;
use crate::run::VenvRunner;
pub use crate::settings::Settings;
pub use crate::ui::{print_error, print_info_1, print_info_2};

#[derive(Debug)]
pub struct Metadata {
    pub dmenv_version: String,
    pub python_platform: String,
    pub python_version: String,
}

#[derive(Debug)]
pub enum PostInstallAction {
    RunSetupPyDevelop,
    None,
}

#[derive(Debug, Copy, Clone)]
pub enum ProcessScriptsMode {
    Safe,
    Override,
}

#[derive(Debug)]
pub struct Context {
    paths: Paths,
    python_info: PythonInfo,
    settings: Settings,
    venv_runner: VenvRunner,
}

fn get_context(cmd: &Command) -> Result<Context, Error> {
    let project_path = if let Some(p) = &cmd.project_path {
        PathBuf::from(p)
    } else {
        look_up_for_project_path()?
    };
    let python_info = PythonInfo::new(&cmd.python_binary)?;
    let python_version = python_info.version.clone();
    let settings = Settings::from_shell(&cmd);
    let paths_resolver = PathsResolver::new(project_path.clone(), python_version, &settings);
    let paths = paths_resolver.paths()?;
    let venv_runner = VenvRunner::new(&project_path, &paths.venv);
    Ok(Context {
        paths,
        python_info,
        settings,
        venv_runner,
    })
}

pub fn run_cmd(cmd: Command) -> Result<(), Error> {
    let context = get_context(&cmd);

    match &cmd.sub_cmd {
        SubCommand::Init {
            name,
            version,
            author,
            no_setup_cfg,
        } => commands::init(cmd.project_path, name, version, author, !no_setup_cfg),
        SubCommand::Install { no_develop } => {
            let post_install_action = if *no_develop {
                PostInstallAction::None
            } else {
                PostInstallAction::RunSetupPyDevelop
            };
            commands::install(&context?, post_install_action)
        }
        SubCommand::ProcessScripts { force } => {
            let mode = if *force {
                ProcessScriptsMode::Override
            } else {
                ProcessScriptsMode::Safe
            };
            process_scripts(&context?, mode)
        }
        SubCommand::Clean {} => commands::clean_venv(&context?),
        SubCommand::Develop {} => commands::develop(&context?),
        SubCommand::Lock {
            python_version,
            sys_platform,
        } => {
            let update_options = UpdateOptions {
                python_version: python_version.clone(),
                sys_platform: sys_platform.clone(),
            };
            commands::update_lock(&context?, update_options)
        }
        SubCommand::BumpInLock { name, version, git } => {
            let bump_type = if *git {
                BumpType::Git
            } else {
                BumpType::Simple
            };
            commands::bump_in_lock(&context?, name, version, bump_type)
        }
        SubCommand::Run { ref cmd, no_exec } => {
            if *no_exec {
                run(&context?, &cmd)
            } else {
                run_and_die(&context?, &cmd)
            }
        }
        SubCommand::ShowDeps {} => show_deps(&context?),
        SubCommand::ShowOutDated {} => show_outdated(&context?),
        SubCommand::ShowVenvPath {} => show_venv_path(&context?),
        SubCommand::ShowVenvBin {} => show_venv_bin_path(&context?),

        SubCommand::Tidy {} => tidy(&cmd, &context?),

        SubCommand::UpgradePip {} => commands::upgrade_pip(&context?),
    }
}

fn process_scripts(context: &Context, mode: ProcessScriptsMode) -> Result<(), Error> {
    operations::scripts::process(&context.paths, mode)
}

// Re-generate a clean lock:
//   - clean the virtualenv
//   - re-create it from scratch, while
//     making sure no package is updated,
//     hence the use of `pip install --constraint`
//     in `self.install_editable_with_constraint()`
//  - re-generate the lock by only keeping existing dependencies:
//    see `operations::lock::tidy()`
fn tidy(cmd: &Command, context: &Context) -> Result<(), Error> {
    commands::clean_venv(&context)?;
    // Re-create a context since we've potenntially just
    // deleted the python we used to clean the previous virtualenv
    let context = get_context(&cmd)?;
    commands::create_venv(&context)?;
    commands::install_editable_with_constraint(&context)?;
    let metadata = commands::metadata(&context);
    let frozen_deps = commands::get_frozen_deps(&context)?;
    let Context { paths, .. } = context;
    operations::lock::tidy(&paths.lock, frozen_deps, &metadata)
}

/// Show the dependencies inside the virtualenv.
// Note: Run `pip list` so we get what's *actually* installed, not just
// the contents of the lock file
fn show_deps(context: &Context) -> Result<(), Error> {
    let Context { venv_runner, .. } = context;
    venv_runner.run(&["python", "-m", "pip", "list"])
}

fn show_outdated(context: &Context) -> Result<(), Error> {
    let Context { venv_runner, .. } = context;
    #[rustfmt::skip]
    let cmd = &[
        "python", "-m", "pip",
        "list", "--outdated",
        "--format", "columns",
    ];
    venv_runner.run(cmd)
}

/// Show the resolved virtualenv path.
//
// See `PathsResolver.paths()` for details
fn show_venv_path(context: &Context) -> Result<(), Error> {
    let Context { paths, .. } = context;
    println!("{}", paths.venv.display());
    Ok(())
}

/// Same has `show_venv_path`, but add the correct subfolder
/// (`bin` on Linux and macOS, `Scripts` on Windows).
fn show_venv_bin_path(context: &Context) -> Result<(), Error> {
    let Context { venv_runner, .. } = context;
    commands::expect_venv(&context)?;
    let bin_path = venv_runner.binaries_path();
    println!("{}", bin_path.display());
    Ok(())
}

fn look_up_for_project_path() -> Result<PathBuf, Error> {
    let mut candidate = std::env::current_dir()
        .map_err(|e| new_error(format!("Could not get current directory: {}", e)))?;
    loop {
        let setup_py_path = candidate.join("setup.py");
        if setup_py_path.exists() {
            return Ok(candidate);
        } else {
            let parent = candidate.parent();
            match parent {
                None => {
                    return Err(new_error(
                        "Could not find setup.py in any of the parent directories".to_string(),
                    ))
                }
                Some(p) => candidate = p.to_path_buf(),
            }
        }
    }
}

/// Run a program from the virtualenv, making sure it dies
/// when we get killed and that the exit code is forwarded
fn run_and_die<T: AsRef<str>>(context: &Context, cmd: &[T]) -> Result<(), Error> {
    let Context { venv_runner, .. } = context;
    commands::expect_venv(&context)?;
    venv_runner.run_and_die(cmd)
}

/// On Windows:
///   - same as run
/// On Linux:
///   - same as run, but create a new process instead of using execv()
// Note: mostly for tests. We want to *check* the return code of
// `dmenv run` and so we need a child process
fn run<T: AsRef<str>>(context: &Context, cmd: &[T]) -> Result<(), Error> {
    let Context { venv_runner, .. } = context;
    commands::expect_venv(&context)?;
    venv_runner.run(cmd)
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_not_in_venv() {
        // If we run `cargo test` from an existing virtualenv, it will get
        // shared by all the tests and most of the integration tests will fail.
        //
        // The goal of _this_ test is to prevent integration tests from running
        // *at all* if we are inside a virtualenv. It works because `cargo test`
        // is clever and does not try to run integration tests when unit tests
        // fail.
        if std::env::var("VIRTUAL_ENV").is_ok() {
            panic!("Please exit virtualenv before running tests");
        }
    }
}
