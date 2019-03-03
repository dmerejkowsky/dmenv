use colored::*;

mod cmd;
mod dependencies;
mod error;
#[cfg(unix)]
mod execv;
mod lock;
mod paths;
mod python_info;
mod settings;
mod venv_manager;
#[cfg(windows)]
mod win_job;

pub use crate::cmd::Command;
use crate::cmd::SubCommand;
pub use crate::cmd::{print_error, print_info_1, print_info_2};
pub use crate::error::Error;
use crate::paths::PathsResolver;
pub use crate::paths::DEV_LOCK_FILENAME;
pub use crate::paths::PROD_LOCK_FILENAME;
use crate::python_info::PythonInfo;
pub use crate::settings::Settings;
use crate::venv_manager::VenvManager;
use crate::venv_manager::{InstallOptions, LockOptions};

pub fn run(cmd: Command, settings: Settings) -> Result<(), Error> {
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
    let python_info = PythonInfo::new(&cmd.python_binary)?;
    let python_version = python_info.version.clone();
    let resolver = PathsResolver::new(project_path, &python_version, settings.clone());
    let paths = resolver.paths()?;
    let venv_manager = VenvManager::new(paths, python_info, settings)?;
    match &cmd.sub_cmd {
        SubCommand::Install { no_develop } => {
            let mut install_options = InstallOptions::default();
            install_options.develop = !no_develop;
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
        SubCommand::ShowVenvBin {} => venv_manager.show_venv_bin_path(),
        SubCommand::UpgradePip {} => venv_manager.upgrade_pip(),
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_not_in_venv() {
        if std::env::var("VIRTUAL_ENV").is_ok() {
            panic!("Please exit virtualenv before running tests");
        }
    }
}
