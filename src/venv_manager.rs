extern crate colored;
use colored::*;

use error::Error;

pub const LOCK_FILE_NAME: &str = "requirements.lock";

pub struct VenvManager {
    working_dir: std::path::PathBuf,
    venv_path: std::path::PathBuf,
    lock_path: std::path::PathBuf,
    setup_py_path: std::path::PathBuf,
    python_binary: std::path::PathBuf,
}

impl VenvManager {
    pub fn new(
        python_binary: std::path::PathBuf,
        python_version: String,
        working_dir: std::path::PathBuf,
    ) -> Result<Self, Error> {
        let lock_path = working_dir.join(LOCK_FILE_NAME);
        let setup_py_path = working_dir.join("setup.py");
        let venv_path = if let Ok(env_var) = std::env::var("VIRTUAL_ENV") {
            std::path::PathBuf::from(env_var)
        } else {
            working_dir.join(".venv").join(&python_version)
        };
        let venv_manager = VenvManager {
            working_dir,
            python_binary,
            venv_path,
            lock_path,
            setup_py_path,
        };
        Ok(venv_manager)
    }

    pub fn clean(&self) -> Result<(), Error> {
        println!(
            "{} Cleaning {}",
            "::".blue(),
            &self.venv_path.to_string_lossy()
        );
        if !self.venv_path.exists() {
            return Ok(());
        }
        std::fs::remove_dir_all(&self.venv_path).map_err(|x| x.into())
    }

    pub fn install(&self) -> Result<(), Error> {
        self.ensure_venv()?;
        self.upgrade_pip()?;

        if !self.lock_path.exists() {
            return Err(Error::new(&format!(
                "{} does not exist. Please run dmenv lock",
                &self.lock_path.to_string_lossy(),
            )));
        }

        self.install_from_lock()
    }

    pub fn run(&self, args: &Vec<String>) -> Result<(), Error> {
        if !self.venv_path.exists() {
            let mut message = format!(
                "The virtualenv in {} does not exist",
                self.venv_path.to_string_lossy().bold()
            );
            message.push_str("\n");
            message.push_str("Please run `dmenv lock` or `dmenv install` to create it");
            return Err(Error::new(&message));
        }
        let cmd = args[0].clone();
        let args: Vec<&str> = args.iter().skip(1).map(|x| x.as_str()).collect();
        self.run_venv_cmd(&cmd, args)
    }

    pub fn lock(&self) -> Result<(), Error> {
        if !self.setup_py_path.exists() {
            return Err(Error::new(
                "setup.py not found. You may want to run `dmenv init` now",
            ));
        }

        self.ensure_venv()?;
        self.upgrade_pip()?;

        println!("{} Generating requirements.txt from setup.py", "::".blue());
        self.install_editable()?;
        self.run_pip_freeze()?;
        Ok(())
    }

    pub fn show(&self) -> Result<(), Error> {
        println!("{}", self.venv_path.to_string_lossy());
        Ok(())
    }

    pub fn init(&self, name: &str, version: &str) -> Result<(), Error> {
        if self.setup_py_path.exists() {
            return Err(Error::new("setup.py already exists. Aborting"));
        }
        let template = include_str!("setup.in.py");
        let template = template.replace("<NAME>", name);
        let template = template.replace("<VERSION>", version);
        std::fs::write(&self.setup_py_path, template)?;
        println!("{} Generated a new setup.py", "::".blue());
        Ok(())
    }

    fn ensure_venv(&self) -> Result<(), Error> {
        if self.venv_path.exists() {
            println!(
                "{} Using existing virtualenv: {}",
                "->".blue(),
                self.venv_path.to_string_lossy()
            );
        } else {
            self.create_venv()?;
        }
        Ok(())
    }

    fn create_venv(&self) -> Result<(), Error> {
        let parent_venv_path = &self.venv_path.parent();
        if parent_venv_path.is_none() {
            return Err(Error::new("venv_path has no parent"));
        }
        let parent_venv_path = parent_venv_path.unwrap();
        println!(
            "{} Creating virtualenv in: {}",
            "::".blue(),
            self.venv_path.to_string_lossy()
        );
        std::fs::create_dir_all(&parent_venv_path)?;
        let venv_path = &self.venv_path.to_string_lossy();
        let args = vec!["-m", "venv", venv_path];
        Self::print_cmd(&self.python_binary.to_string_lossy(), &args);
        let status = std::process::Command::new(&self.python_binary)
            .current_dir(&self.working_dir)
            .args(&args)
            .status()?;
        if !status.success() {
            return Err(Error::new("Failed to create virtualenv"));
        }
        Ok(())
    }

    fn run_pip_freeze(&self) -> Result<(), Error> {
        let python = self.get_path_in_venv("python")?;
        let args = vec!["-m", "pip", "freeze", "--exclude-editable"];
        let python_str = python.to_string_lossy().to_string();
        Self::print_cmd(&python_str, &args);
        let command = std::process::Command::new(python)
            .current_dir(&self.working_dir)
            .args(args)
            .output()?;
        if !command.status.success() {
            return Err(Error::new(&format!(
                "pip freeze failed: {}",
                String::from_utf8_lossy(&command.stderr)
            )));
        }
        std::fs::write(&self.lock_path, &command.stdout)?;
        println!(
            "{} Requirements written to {}",
            "::".blue(),
            self.lock_path.to_string_lossy()
        );
        Ok(())
    }

    fn install_from_lock(&self) -> Result<(), Error> {
        let as_str = &self.lock_path.to_string_lossy();
        let args = vec![
            "-m",
            "pip",
            "install",
            "--requirement",
            as_str,
            "--editable",
            ".[dev]",
        ];
        self.run_venv_cmd("python", args)
    }

    pub fn upgrade_pip(&self) -> Result<(), Error> {
        let args = vec!["-m", "pip", "install", "pip", "--upgrade"];
        self.run_venv_cmd("python", args)
    }

    fn install_editable(&self) -> Result<(), Error> {
        // tells pip to run `setup.py develop` (that's -e), and
        // install the dev requirements too
        let args = vec!["-m", "pip", "install", "-e", ".[dev]"];
        self.run_venv_cmd("python", args)
    }

    fn run_venv_cmd(&self, name: &str, args: Vec<&str>) -> Result<(), Error> {
        let bin_path = &self.get_path_in_venv(name)?;
        Self::print_cmd(&bin_path.to_string_lossy(), &args);
        let command = std::process::Command::new(bin_path)
            .args(args)
            .current_dir(&self.working_dir)
            .status()?;
        if !command.success() {
            return Err(Error::new("command failed"));
        }

        Ok(())
    }

    fn get_path_in_venv(&self, name: &str) -> Result<std::path::PathBuf, Error> {
        if !self.venv_path.exists() {
            return Err(Error::new(&format!(
                "virtualenv in {} does not exist",
                &self.venv_path.to_string_lossy()
            )));
        }

        #[cfg(not(windows))]
        let binaries_subdirs = "bin";
        #[cfg(not(windows))]
        let suffix = "";

        #[cfg(windows)]
        let binaries_subdirs = "Scripts";
        #[cfg(windows)]
        let suffix = ".exe";

        let name = format!("{}{}", name, suffix);
        let path = self.venv_path.join(binaries_subdirs).join(name);
        if !path.exists() {
            return Err(Error::new(&format!(
                "Cannot run: '{}' does not exist",
                &path.to_string_lossy()
            )));
        }
        Ok(path)
    }

    fn print_cmd(bin_path: &str, args: &Vec<&str>) {
        println!(
            "{} running {} {}",
            "->".blue(),
            bin_path.bold(),
            args.join(" ")
        );
    }
}
