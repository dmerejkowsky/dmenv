use colored::*;
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
pub use crate::paths::{DEV_LOCK_FILENAME, PROD_LOCK_FILENAME};
use crate::project::{PostInstallAction, ProcessScriptsMode, Project};
use crate::python_info::PythonInfo;
pub use crate::settings::Settings;

        PathBuf::from(p)
    } else {
    };
}

pub fn run(cmd: Command) -> Result<(), Error> {
    let settings = Settings::from_shell(&cmd);

    if let SubCommand::Init {
        name,
        version,
        author,
        no_setup_cfg,
    } = &cmd.sub_cmd
    {
        return init(cmd.project_path, name, version, author, !no_setup_cfg);
    }

    let project_path = if let Some(p) = cmd.project_path {
        PathBuf::from(p)
    } else {
        look_up_for_project_path()?
    };
    let python_info = PythonInfo::new(&cmd.python_binary)?;
    let project = Project::new(project_path, python_info, settings)?;

    match &cmd.sub_cmd {
        SubCommand::Install { no_develop } => {
            let post_install_action = if *no_develop {
                PostInstallAction::None
            } else {
                PostInstallAction::RunSetupPyDevelop
            };
            project.install(post_install_action)
        }
        SubCommand::ProcessScripts { force } => {
            let mode = if *force {
                ProcessScriptsMode::Override
            } else {
                ProcessScriptsMode::Safe
            };
            project.process_scripts(mode)
        }
        SubCommand::Clean {} => project.clean_venv(),
        SubCommand::Develop {} => project.develop(),
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
