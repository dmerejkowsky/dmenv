extern crate appdirs;

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
        let name = current_dir.file_name();
        if name.is_none() {
            // Dude, wtf?
            return Err(Error::new("current directory has no filename"));
        }
        let name = name.unwrap();
        let data_dir = appdirs::user_data_dir(Some("dmenv"), None, false);
        // The type is Result<PathBuf, ()> I blame upstream
        if data_dir.is_err() {
            return Err(Error::new(
                "appdirs::user_data_dir() failed. That's all we know",
            ));
        }
        let data_dir = data_dir.unwrap();
        let venv_path = data_dir.join("venvs").join(name);
        let requirements_lock_path = current_dir.join("requirements.lock");
        let app = App {
            venv_path,
            requirements_lock_path,
        };
        Ok(app)
    }

    pub fn clean(&self) -> Result<(), Error> {
        println!("Cleaning {}", &self.venv_path.to_string_lossy());
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

        self.run_pip_install()
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
        println!("Generating requirements.txt from setup.py");

        self.install_editable()?;
        self.run_pip_freeze()?;
        Ok(())
    }

    fn create_venv(&self) -> Result<(), Error> {
        let parent_venv_path = &self.venv_path.parent();
        if parent_venv_path.is_none() {
            return Err(Error::new("venv_path has no parent"));
        }
        let parent_venv_path = parent_venv_path.unwrap();
        println!(
            "Creating virtualenv in: {}",
            self.venv_path.to_string_lossy()
        );
        std::fs::create_dir_all(&parent_venv_path)?;
        let status = std::process::Command::new("python")
            .args(&["-m", "venv", &self.venv_path.to_string_lossy()])
            .status()?;
        if !status.success() {
            println!("Failed to create virtualenv");
        }
        // Always upgrade pip right after creation. We use 'venv' from the
        // stdlib, so the pip in the virtualenv is the one bundle with python
        // which in all likelyhood is old
        self.upgrade_pip()
    }

    fn run_pip_freeze(&self) -> Result<(), Error> {
        let command = &mut self.build_command("python")?;

        let command = command
            .args(&["-m", "pip", "freeze", "--exclude-editable"])
            .output()?;
        if !command.status.success() {
            return Err(Error::new("pip freeze failed"));
        }
        std::fs::write("requirements.lock", &command.stdout)?;
        println!("Requirements written to requirements.lock");
        Ok(())
    }

    fn run_pip_install(&self) -> Result<(), Error> {
        let as_str = &self.requirements_lock_path.to_string_lossy();
        let args = vec!["-m", "pip", "install", "--requirement", as_str];
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
        let command = &mut self.build_command(&name)?;
        let command = command.args(args).status()?;
        if !command.success() {
            return Err(Error::new("command failed"));
        }

        Ok(())
    }

    fn build_command(&self, name: &str) -> Result<std::process::Command, Error> {
        if !self.venv_path.exists() {
            return Err(Error::new(&format!(
                "virtualenv in '{}' does not exist",
                &self.venv_path.to_string_lossy()
            )));
        }

        // TODO: on Windows this is `Scripts`, not `bin`
        let cmd_path = self.venv_path.join("bin").join(name);
        if !cmd_path.exists() {
            return Err(Error::new(&format!(
                "Cannot run: '{}' does not exist",
                &cmd_path.to_string_lossy()
            )));
        }
        Ok(std::process::Command::new(cmd_path))
    }
}
