extern crate colored;
#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate structopt;

mod config;
mod error;
mod options;
mod pythons_manager;
mod venv_manager;

pub use config::ConfigHandler;
pub use error::Error;
use options::Command;
pub use options::Options;
use pythons_manager::PythonsManager;
use venv_manager::VenvManager;
pub use venv_manager::LOCK_FILE_NAME;

fn run_venv_manager(options: Options) -> Result<(), Error> {
    let venv_manager = VenvManager::new(
        &options.python_version,
        options.cfg_path.clone(),
        options.working_dir.clone(),
    )?;
    match &options.cmd {
        Command::Install {} => venv_manager.install(),
        Command::Clean {} => venv_manager.clean(),
        Command::Init { name, version } => venv_manager.init(&name, &version),
        Command::Lock {} => venv_manager.lock(),
        Command::Run { ref cmd } => venv_manager.run(cmd),
        Command::Show {} => venv_manager.show(),
        Command::UpgradePip {} => venv_manager.upgrade_pip(),
        _ => Ok(()),
    }
}

fn run_pythons_manager(
    cfg_path: Option<String>,
    python_cmd: options::PythonCommand,
) -> Result<(), Error> {
    let config_handler = ConfigHandler::new(cfg_path)?;
    let pythons_manager = PythonsManager::new(config_handler);
    match python_cmd {
        options::PythonCommand::Add { version, path } => pythons_manager.add(&version, &path)?,
        options::PythonCommand::Remove { version } => pythons_manager.remove(&version)?,
        options::PythonCommand::List {} => pythons_manager.list()?,
    }
    Ok(())
}

pub fn run(options: Options) -> Result<(), Error> {
    if let Command::Pythons { pythons_cmd } = options.cmd {
        run_pythons_manager(options.cfg_path, pythons_cmd)
    } else {
        run_venv_manager(options)
    }
}
