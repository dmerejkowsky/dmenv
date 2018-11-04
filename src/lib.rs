extern crate colored;
extern crate serde;
extern crate serde_derive;
extern crate structopt;
use colored::*;

mod cmd;
mod error;
mod python_info;
mod venv_manager;

use python_info::PythonInfo;
pub use cmd::Command;
use cmd::SubCommand;
pub use error::Error;
use venv_manager::VenvManager;
pub use venv_manager::LOCK_FILE_NAME;


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
    let venv_manager =
        VenvManager::new(working_dir, python_info)?;
    match &cmd.sub_cmd {
        SubCommand::Install {} => venv_manager.install(),
        SubCommand::Clean {} => venv_manager.clean(),
        SubCommand::Init { name, version } => venv_manager.init(&name, &version),
        SubCommand::Lock {} => venv_manager.lock(),
        SubCommand::Run { ref cmd } => venv_manager.run(cmd),
        SubCommand::Show {} => venv_manager.show(),
        SubCommand::UpgradePip {} => venv_manager.upgrade_pip(),
    }
}
