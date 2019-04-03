use std::path::PathBuf;

use colored::*;

use crate::error::Error;
use crate::operations;

pub struct VenvRunner {
    project_path: PathBuf,
    venv_path: PathBuf,
}

impl VenvRunner {
    pub fn new(project_path: &PathBuf, venv_path: &PathBuf) -> Self {
        VenvRunner {
            project_path: project_path.to_path_buf(),
            venv_path: venv_path.to_path_buf(),
        }
    }
    pub fn run(&self, binary: &str, args: Vec<&str>) -> Result<(), Error> {
        operations::venv::expect(&self.venv_path)?;

        let binary_path = self.resolve_path(binary);
        if !binary_path.exists() {
            return Err(Error::Other {
                message: format!("Cannot run: '{}' does not exist", &binary_path.display()),
            });
        };
        run(&self.project_path, &binary_path, args)
    }

    pub fn get_output(&self, binary: &str, args: Vec<&str>) -> Result<String, Error> {
        let binary_path = self.resolve_path(binary);
        if !binary_path.exists() {
            return Err(Error::Other {
                message: format!("Cannot run: '{}' does not exist", &binary_path.display()),
            });
        };
        get_output(&self.project_path, &binary_path, args)
    }

    pub fn resolve_path(&self, binary: &str) -> PathBuf {
        #[cfg(windows)]
        let suffix = ".exe";
        #[cfg(not(windows))]
        let suffix = "";

        let binary = format!("{}{}", binary, suffix);

        let binaries_path = self.binaries_path();
        self.venv_path.join(binaries_path).join(binary)
    }

    pub fn binaries_path(&self) -> PathBuf {
        #[cfg(not(windows))]
        let subdir = "bin";

        #[cfg(windows)]
        let subdir = "Scripts";
        self.venv_path.join(subdir)
    }
}

pub fn run(working_path: &PathBuf, binary_path: &PathBuf, args: Vec<&str>) -> Result<(), Error> {
    println!(
        "{} {} {}",
        "$".blue(),
        binary_path.display(),
        args.join(" ")
    );
    let command = std::process::Command::new(binary_path)
        .args(args)
        .current_dir(working_path)
        .status();
    let command = command.map_err(|e| Error::ProcessWaitError { io_error: e })?;
    if !command.success() {
        return Err(Error::Other {
            message: "command failed".to_string(),
        });
    }
    Ok(())
}

pub fn get_output(
    working_path: &PathBuf,
    binary_path: &PathBuf,
    args: Vec<&str>,
) -> Result<String, Error> {
    let cmd_str = format!("{} {}", binary_path.display(), args.join(" "));
    let command = std::process::Command::new(binary_path)
        .args(args)
        .current_dir(working_path)
        .output();

    let command = command.map_err(|e| Error::ProcessOutError { io_error: e })?;
    if !command.status.success() {
        return Err(Error::Other {
            message: format!(
                "`{}` failed\n: {}",
                cmd_str,
                String::from_utf8_lossy(&command.stderr)
            ),
        });
    }
    Ok(String::from_utf8_lossy(&command.stdout).to_string())
}
