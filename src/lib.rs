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
use crate::operations::{InitOptions, LockOptions};
pub use crate::paths::{DEV_LOCK_FILENAME, PROD_LOCK_FILENAME};
use crate::project::{PostInstallAction, Project};
use crate::python_info::PythonInfo;
pub use crate::settings::Settings;

pub fn run(cmd: Command) -> Result<(), Error> {
    let settings = Settings::from_shell(&cmd);
    let project_path = if let Some(project_path) = cmd.project_path {
        PathBuf::from(project_path)
    } else {
        std::env::current_dir()
            .map_err(|e| new_error(&format!("Could not get current directory: {}", e)))?
    };
    // Perform additional sanity checks when using `dmenv run`
    // TODO: try and handle this using StructOpt instead
    if let SubCommand::Run { ref cmd, .. } = cmd.sub_cmd {
        if cmd.is_empty() {
            return Err(new_error(&format!(
                "Missing argument after '{}'",
                "run".green()
            )));
        }
    }
    let python_info = PythonInfo::new(&cmd.python_binary)?;
    let mut project = Project::new(project_path, python_info, settings)?;

    // Note: keep the `match()` here so that we know every variant of the SubCommand
    // enum is handled.
    match &cmd.sub_cmd {
        SubCommand::Install {
            no_develop,
            system_site_packages,
        } => {
            if *system_site_packages {
                project.use_system_site_packages()
            }
            let post_install_action = if *no_develop {
                PostInstallAction::None
            } else {
                PostInstallAction::RunSetupPyDevelop
            };
            project.install(post_install_action)
        }
        SubCommand::Clean {} => project.clean(),
        SubCommand::Develop {} => project.develop(),
        SubCommand::Init {
            name,
            version,
            author,
            no_setup_cfg,
        } => {
            let mut options = InitOptions::new(name, version, author);
            if *no_setup_cfg {
                options.no_setup_cfg()
            };
            project.init(&options)
        }
        SubCommand::Lock {
            python_version,
            sys_platform,
            system_site_packages,
        } => {
            let lock_options = LockOptions {
                python_version: python_version.clone(),
                sys_platform: sys_platform.clone(),
            };
            if *system_site_packages {
                project.use_system_site_packages();
            }
            project.lock(&lock_options)
        }
        SubCommand::BumpInLock { name, version, git } => project.bump_in_lock(name, version, *git),
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

        SubCommand::UpgradePip {} => project.upgrade_pip(),
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
