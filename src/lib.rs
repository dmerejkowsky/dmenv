use colored::*;

mod cmd;
mod dependencies;
mod error;
#[cfg(unix)]
mod execv;
mod lock;
mod python_info;
mod venv_manager;
#[cfg(windows)]
mod win_job;

pub use crate::cmd::Command;
use crate::cmd::SubCommand;
pub use crate::cmd::{print_error, print_info_1, print_info_2};
pub use crate::error::Error;
use crate::python_info::PythonInfo;
use crate::venv_manager::VenvManager;
pub use crate::venv_manager::LOCK_FILE_NAME;
use crate::venv_manager::{InstallOptions, LockOptions};

pub fn run(cmd: Command) -> Result<(), Error> {
    let project_path = if let Some(project_path) = cmd.project_path {
        std::path::PathBuf::from(project_path)
    } else {
        std::env::current_dir().map_err(|e| Error::Other {
            message: format!("Could not get current directory: {}", e),
        })?
    };
    if let SubCommand::Run { ref cmd, .. } = cmd.sub_cmd {
        if cmd.is_empty() {
            return Err(Error::Other {
                message: format!("Missing argument after '{}'", "run".green()),
            });
        }
    }
    let mut python_info = PythonInfo::new(&cmd.python_binary)?;
    if std::env::var("DMENV_NO_VENV_STDLIB").is_ok() {
        python_info.venv_from_stdlib = false;
    }
    let venv_manager = VenvManager::new(project_path, python_info)?;
    match &cmd.sub_cmd {
        SubCommand::Install {
            no_develop,
            no_upgrade_pip,
        } => {
            let mut install_options = InstallOptions::default();
            install_options.develop = !no_develop;
            install_options.upgrade_pip = !no_upgrade_pip;
            venv_manager.install(&install_options)
        }
        SubCommand::Clean {} => venv_manager.clean(),
        SubCommand::Develop {} => venv_manager.develop(),
        SubCommand::Init {
            name,
            version,
            author,
        } => venv_manager.init(&name, &version, author),
        SubCommand::Lock {
            python_version,
            sys_platform,
        } => {
            let lock_options = LockOptions {
                python_version: python_version.clone(),
                sys_platform: sys_platform.clone(),
            };
            venv_manager.lock(&lock_options)
        }
        SubCommand::BumpInLock { name, version, git } => {
            venv_manager.bump_in_lock(name, version, *git)
        }
        SubCommand::Run { ref cmd, no_exec } => {
            if *no_exec {
                venv_manager.run_no_exec(cmd)
            } else {
                venv_manager.run(cmd)
            }
        }
        SubCommand::ShowDeps {} => venv_manager.show_deps(),
        SubCommand::ShowVenvPath {} => venv_manager.show_venv_path(),
        SubCommand::UpgradePip {} => venv_manager.upgrade_pip(),
    }
}
