use std::path::PathBuf;

mod cmd;
mod dependencies;
mod error;
#[cfg(unix)]
mod execv;
mod lock;
mod operations;
mod paths;
mod project;
mod python_info;
mod run;
mod settings;
#[cfg(windows)]
mod win_job;

pub use crate::cmd::Command;
use crate::cmd::SubCommand;
pub use crate::cmd::{print_error, print_info_1, print_info_2};
pub use crate::error::*;
use crate::lock::BumpType;
use crate::operations::{InitOptions, UpdateOptions};
use crate::paths::{Paths, PathsResolver};
pub use crate::paths::{DEV_LOCK_FILENAME, PROD_LOCK_FILENAME};
use crate::project::{PostInstallAction, ProcessScriptsMode, Project};
use crate::python_info::PythonInfo;
use crate::run::VenvRunner;
pub use crate::settings::Settings;

struct Context {
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

pub fn run(cmd: Command) -> Result<(), Error> {
    if let SubCommand::Init {
        name,
        version,
        author,
        no_setup_cfg,
    } = &cmd.sub_cmd
    {
        return init(cmd.project_path, name, version, author, !no_setup_cfg);
    }

    let python_info = PythonInfo::new(&cmd.python_binary)?;
        PathBuf::from(p)
    } else {
        look_up_for_project_path()?
    };
    let settings = Settings::from_shell(&cmd);
    let project = Project::new(project_path, python_info, settings)?;

    let context = get_context(&cmd)?;

    match &cmd.sub_cmd {
        SubCommand::Install { no_develop } => {
            let post_install_action = if *no_develop {
                PostInstallAction::None
            } else {
                PostInstallAction::RunSetupPyDevelop
            };
            install(&context, post_install_action)
        }
        SubCommand::ProcessScripts { force } => {
            let mode = if *force {
                ProcessScriptsMode::Override
            } else {
                ProcessScriptsMode::Safe
            };
            process_scripts(&context, mode)
        }
        SubCommand::Clean {} => clean_venv(&context),
        SubCommand::Develop {} => develop(&context),
        SubCommand::Lock {
            python_version,
            sys_platform,
        } => {
            let update_options = UpdateOptions {
                python_version: python_version.clone(),
                sys_platform: sys_platform.clone(),
            };
            project.update_lock(update_options)
        }
        SubCommand::BumpInLock { name, version, git } => {
            let bump_type = if *git {
                BumpType::Git
            } else {
                BumpType::Simple
            };
            project.bump_in_lock(name, version, bump_type)
        }
        SubCommand::Run { ref cmd, no_exec } => {
            if *no_exec {
                project.run(&cmd)
            } else {
                project.run_and_die(&cmd)
            }
        }
        SubCommand::ShowDeps {} => project.show_deps(),
        SubCommand::ShowOutDated {} => project.show_outdated(),
        SubCommand::ShowVenvPath {} => project.show_venv_path(),
        SubCommand::ShowVenvBin {} => project.show_venv_bin_path(),

        SubCommand::Tidy {} => project.tidy(),

        SubCommand::UpgradePip {} => project.upgrade_pip(),
        _ => unimplemented!("Subcommand {:?} not handled", cmd.sub_cmd),
    }
}

fn init(
    project_path: Option<String>,
    name: &str,
    version: &str,
    author: &Option<String>,
    setup_cfg: bool,
) -> Result<(), Error> {
    let init_path = if let Some(p) = project_path {
        PathBuf::from(p)
    } else {
        std::env::current_dir().map_err(|e| Error::NoWorkingDirectory { io_error: e })?
    };

    let mut init_options = InitOptions::new(name.to_string(), version.to_string());
    if !setup_cfg {
        init_options.no_setup_cfg();
    };
    if let Some(author) = author {
        init_options.author(&author);
    }
    operations::init(&init_path, &init_options)
}

fn install(context: &Context, post_install_action: PostInstallAction) -> Result<(), Error> {
    let Context {
        settings, paths, ..
    } = context;
    if settings.production {
        print_info_1("Preparing project for production")
    } else {
        print_info_1("Preparing project for development")
    };
    let lock_path = &paths.lock;
    if !lock_path.exists() {
        return Err(Error::MissingLock {
            expected_path: lock_path.to_path_buf(),
        });
    }

    ensure_venv(context)?;
    install_from_lock(context)?;

    match post_install_action {
        PostInstallAction::RunSetupPyDevelop => develop(context)?,
        PostInstallAction::None => (),
    }
    Ok(())
}

fn process_scripts(context: &Context, mode: ProcessScriptsMode) -> Result<(), Error> {
    operations::scripts::process(&context.paths, mode)
}

fn ensure_venv(context: &Context) -> Result<(), Error> {
    let Context { paths, .. } = context;
    if paths.venv.exists() {
        print_info_2(&format!(
            "Using existing virtualenv: {}",
            paths.venv.display()
        ));
    } else {
        create_venv(context)?;
    }
    Ok(())
}

fn create_venv(context: &Context) -> Result<(), Error> {
    let Context {
        paths,
        python_info,
        settings,
        ..
    } = context;
    // TODO: venv::create(context)
    operations::venv::create(&paths.venv, python_info, settings)
}

fn clean_venv(context: &Context) -> Result<(), Error> {
    let Context { paths, .. } = context;
    operations::venv::clean(paths.venv.clone())
}

fn install_from_lock(context: &Context) -> Result<(), Error> {
    let Context {
        paths, venv_runner, ..
    } = context;
    let lock_path = &paths.lock;
    print_info_2(&format!(
        "Installing dependencies from {}",
        lock_path.display()
    ));
    // Since we'll be running the command using self.paths.project
    // as working directory, we must use the *relative* lock file
    // name when calling `pip install`.
    let lock_name = paths
        .lock
        .file_name()
        .unwrap_or_else(|| panic!("self.path.lock has no filename component"));

    let as_str = lock_name.to_string_lossy();
    let cmd = &["python", "-m", "pip", "install", "--requirement", &as_str];
    venv_runner.run(cmd)
}

/// Runs `python setup.py` develop. Also called by `install` (unless InstallOptions.develop is false)
// Note: `lock()` will use `pip install --editable .` to achieve the same effect
fn develop(context: &Context) -> Result<(), Error> {
    let Context {
        paths, venv_runner, ..
    } = context;
    print_info_2("Running setup_py.py develop");
    if !&paths.setup_py.exists() {
        return Err(Error::MissingSetupPy {});
    }

    venv_runner.run(&["python", "setup.py", "develop", "--no-deps"])
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
