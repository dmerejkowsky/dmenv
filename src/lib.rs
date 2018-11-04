extern crate colored;
extern crate serde;
extern crate serde_derive;
extern crate structopt;
use colored::*;

mod cmd;
mod error;
mod venv_manager;

pub use cmd::Command;
use cmd::SubCommand;
pub use error::Error;
use venv_manager::VenvManager;
pub use venv_manager::LOCK_FILE_NAME;

fn get_python_binary(requested_python: &Option<String>) -> Result<std::path::PathBuf, Error> {
    if let Some(python) = requested_python {
        return Ok(std::path::PathBuf::from(python));
    }

    let python3 = which::which("python3");
    if python3.is_ok() {
        return Ok(python3.unwrap());
    }

    // Python3 may be called 'python', for instance on Windows
    Ok(which::which("python")?)
}

fn get_python_version(python_binary: &std::path::PathBuf) -> Result<String, Error> {
    let command = std::process::Command::new(python_binary)
        .args(&["--version"])
        .output()?;
    if !command.status.success() {
        return Err(Error::new(&format!(
            "python --version failed: {}",
            String::from_utf8_lossy(&command.stderr)
        )));
    }
    let out = String::from_utf8_lossy(&command.stdout);
    let out = out.trim();
    let version = out.replace("Python ", "");
    Ok(version)
}

pub fn run(cmd: Command) -> Result<(), Error> {
    let working_dir = if let Some(cwd) = cmd.working_dir {
        std::path::PathBuf::from(cwd)
    } else {
        std::env::current_dir()?
    };
    if let SubCommand::Run { ref cmd } = cmd.sub_cmd {
        if cmd.len() == 0 {
            return Err(Error::new(&format!("Missing argument after '{}'", "run".green())));
        }
    }
    let python_binary = get_python_binary(&cmd.python_binary)?;
    let python_version = get_python_version(&python_binary)?;
    let venv_manager = VenvManager::new(python_binary, python_version, working_dir)?;
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
