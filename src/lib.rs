extern crate colored;
extern crate serde;
extern crate serde_derive;
extern crate structopt;
use colored::*;

mod cmd;
mod error;
mod python_info;
mod venv_manager;

pub use cmd::Command;
use cmd::SubCommand;
pub use error::Error;
use python_info::PythonInfo;
use venv_manager::VenvManager;
pub use venv_manager::LOCK_FILE_NAME;
use venv_manager::{InstallOptions, LockOptions};

pub fn run(cmd: Command) -> Result<(), Error> {
    let working_dir = if let Some(cwd) = cmd.working_dir {
        std::path::PathBuf::from(cwd)
    } else {
        std::env::current_dir()?
    };
    if let SubCommand::Run { ref cmd } = cmd.sub_cmd {
        if cmd.len() == 0 {
            return Err(Error::new(&format!(
                "Missing argument after '{}'",
                "run".green()
            )));
        }
    }
    let python_info = PythonInfo::new(&cmd.python_binary)?;
    let venv_manager = VenvManager::new(working_dir, python_info)?;
    match &cmd.sub_cmd {
        SubCommand::Install {
            no_develop,
            no_upgrade_pip,
        } => {
            let mut install_options = InstallOptions::default();
            install_options.develop = !no_develop;
            install_options.upgrade_pip = !no_upgrade_pip;
            venv_manager.install(install_options)
        }
        SubCommand::Clean {} => venv_manager.clean(),
        SubCommand::Develop {} => venv_manager.develop(),
        SubCommand::Init {
            name,
            version,
            author,
        } => venv_manager.init(&name, &version, author),
        SubCommand::Lock { clean } => {
            let mut lock_options = LockOptions::default();
            lock_options.clean = *clean;
            venv_manager.lock(lock_options)
        }
        SubCommand::Run { ref cmd } => venv_manager.run(cmd),
        SubCommand::ShowDeps {} => venv_manager.show_deps(),
        SubCommand::ShowVenvPath {} => venv_manager.show_venv_path(),
        SubCommand::UpgradePip {} => venv_manager.upgrade_pip(),
    }
}
