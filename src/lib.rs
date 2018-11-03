extern crate colored;
#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate structopt;

mod cmd;
mod config;
mod error;
mod pythons_manager;
mod venv_manager;

pub use config::ConfigHandler;
pub use error::Error;
use cmd::SubCommand;
use cmd::PythonsCommand;
pub use cmd::Command;
use pythons_manager::PythonsManager;
use venv_manager::VenvManager;
pub use venv_manager::LOCK_FILE_NAME;

fn run_venv_manager(cmd: Command) -> Result<(), Error> {
    let venv_manager = VenvManager::new(
        &cmd.python_version,
        cmd.cfg_path.clone(),
        cmd.working_dir.clone(),
    )?;
    match &cmd.sub_cmd {
        SubCommand::Install {} => venv_manager.install(),
        SubCommand::Clean {} => venv_manager.clean(),
        SubCommand::Init { name, version } => venv_manager.init(&name, &version),
        SubCommand::Lock {} => venv_manager.lock(),
        SubCommand::Run { ref cmd } => venv_manager.run(cmd),
        SubCommand::Show {} => venv_manager.show(),
        SubCommand::UpgradePip {} => venv_manager.upgrade_pip(),
        _ => Ok(()),
    }
}

fn run_pythons_manager(
    cfg_path: Option<String>,
    python_cmd: PythonsCommand,
) -> Result<(), Error> {
    let config_handler = ConfigHandler::new(cfg_path)?;
    let pythons_manager = PythonsManager::new(config_handler);
    match python_cmd {
        PythonsCommand::Add { version, path } => pythons_manager.add(&version, &path)?,
        PythonsCommand::Remove { version } => pythons_manager.remove(&version)?,
        PythonsCommand::List {} => pythons_manager.list()?,
    }
    Ok(())
}

pub fn run(cmd: Command) -> Result<(), Error> {
    if let SubCommand::Pythons { pythons_cmd } = cmd.sub_cmd {
        run_pythons_manager(cmd.cfg_path, pythons_cmd)
    } else {
        run_venv_manager(cmd)
    }
}
