extern crate colored;
use colored::*;

pub struct App {
    venv_path: std::path::PathBuf,
    requirements_lock_path: std::path::PathBuf,
}

#[derive(Debug)]
pub struct Error {
    description: String,
}

impl Error {
    pub fn new(description: &str) -> Error {
        Error {
            description: String::from(description),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Error {
        Error::new(&format!("I/O error: {}", error))
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", &self.description)
    }
}

impl App {
    pub fn new() -> Result<Self, Error> {
        let current_dir = std::env::current_dir()?;
        let venv_path = current_dir.join(".venv");
        let requirements_lock_path = current_dir.join("requirements.lock");
        let app = App {
            venv_path,
            requirements_lock_path,
        };
        Ok(app)
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
        if !self.venv_path.exists() {
            self.create_venv()?;
        }

        if !self.requirements_lock_path.exists() {
            return Err(Error::new(&format!(
                "{} does not exist. Please run dmenv freeze",
                &self.requirements_lock_path.to_string_lossy(),
            )));
        }

        self.install_from_lock()
    }

    pub fn run(&self, args: Vec<String>) -> Result<(), Error> {
        let bin_path = &self.venv_path.join("bin").join(&args[0]);
        let command = std::process::Command::new(bin_path)
            .args(&args[1..])
            .status()?;
        if !command.success() {
            return Err(Error::new("command failed"));
        }
        Ok(())
    }

    pub fn freeze(&self) -> Result<(), Error> {
        if !self.venv_path.exists() {
            self.create_venv()?;
        }

        println!("{} Generating requirements.txt from setup.py", "::".blue());
        self.install_editable()?;
        self.run_pip_freeze()?;
        Ok(())
    }

    pub fn show(&self) -> Result<(), Error> {
        println!("{}", self.venv_path.to_string_lossy());
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
        let status = std::process::Command::new("python")
            .args(&["-m", "venv", &self.venv_path.to_string_lossy()])
            .status()?;
        if !status.success() {
            return Err(Error::new("Failed to create virtualenv"));
        }
        // Always upgrade pip right after creation. We use 'venv' from the
        // stdlib, so the pip in the virtualenv is the one bundle with python
        // which in all likelyhood is old
        self.upgrade_pip()
    }

    fn run_pip_freeze(&self) -> Result<(), Error> {
        let python = self.get_path_in_venv("python")?;
        let args = vec!["-m", "pip", "freeze", "--exclude-editable"];
        Self::print_cmd(python.to_path_buf(), &args);
        let command = std::process::Command::new(python).args(args).output()?;
        if !command.status.success() {
            return Err(Error::new("pip freeze failed"));
        }
        std::fs::write("requirements.lock", &command.stdout)?;
        println!("{} Requirements written to requirements.lock", "::".blue());
        Ok(())
    }

    fn install_from_lock(&self) -> Result<(), Error> {
        let as_str = &self.requirements_lock_path.to_string_lossy();
        let args = vec![
            "-m",
            "pip",
            "install",
            "--requirement",
            as_str,
            "-e",
            ".[dev]",
        ];
        self.run_venv_bin("python", args)
    }

    pub fn upgrade_pip(&self) -> Result<(), Error> {
        let args = vec!["-m", "pip", "install", "pip", "--upgrade"];
        self.run_venv_bin("python", args)
    }

    fn install_editable(&self) -> Result<(), Error> {
        // tells pip to run `setup.py develop` (that's -e), and
        // install the dev requirements too
        let args = vec!["-m", "pip", "install", "-e", ".[dev]"];
        self.run_venv_bin("python", args)
    }

    fn run_venv_bin(&self, name: &str, args: Vec<&str>) -> Result<(), Error> {
        let bin_path = &self.get_path_in_venv(name)?;
        Self::print_cmd(bin_path.to_path_buf(), &args);
        let command = std::process::Command::new(bin_path).args(args).status()?;
        if !command.success() {
            return Err(Error::new("command failed"));
        }

        Ok(())
    }

    fn get_path_in_venv(&self, name: &str) -> Result<std::path::PathBuf, Error> {
        if !self.venv_path.exists() {
            return Err(Error::new(&format!(
                "virtualenv in '{}' does not exist",
                &self.venv_path.to_string_lossy()
            )));
        }

        // TODO: on Windows this is `Scripts`, not `bin`
        let path = self.venv_path.join("bin").join(name);
        if !path.exists() {
            return Err(Error::new(&format!(
                "Cannot run: '{}' does not exist",
                &path.to_string_lossy()
            )));
        }
        Ok(path)
    }

    fn print_cmd(bin_path: std::path::PathBuf, args: &Vec<&str>) {
        println!(
            "{} running {} {}",
            "->".blue(),
            bin_path.to_string_lossy().bold(),
            args.join(" ")
        );
    }
}
