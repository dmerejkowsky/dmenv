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

pub enum BumpType {
    Git,
    Simple,
}

#[derive(Default, Debug)]
/// Represents options passed to `dmenv lock`,
/// see `cmd::SubCommand::Lock`
pub struct UpdateOptions {
    pub python_version: Option<String>,
    pub sys_platform: Option<String>,
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

        SubCommand::Clean {} => commands::clean_venv(&context?),
        SubCommand::Develop {} => commands::develop(&context?),
        SubCommand::UpgradePip {} => commands::upgrade_pip(&context?),

        SubCommand::ProcessScripts { force } => {
            let mode = if *force {
                ProcessScriptsMode::Override
            } else {
                ProcessScriptsMode::Safe
            };
            commands::process_scripts(&context?, mode)
        }

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
                commands::run(&context?, &cmd)
            } else {
                commands::run_and_die(&context?, &cmd)
            }
        }

        SubCommand::ShowDeps {} => commands::show_deps(&context?),
        SubCommand::ShowOutDated {} => commands::show_outdated(&context?),
        SubCommand::ShowVenvPath {} => commands::show_venv_path(&context?),
        SubCommand::ShowVenvBin {} => commands::show_venv_bin_path(&context?),

        SubCommand::Tidy {} => commands::tidy(&cmd, &context?),
    }
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
