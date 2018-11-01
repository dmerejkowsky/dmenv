extern crate colored;
#[macro_use]
extern crate serde_derive;
extern crate serde;
use colored::*;
use std::collections::BTreeMap as Map;

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

impl From<toml::de::Error> for Error {
    fn from(error: toml::de::Error) -> Error {
        Error::new(&format!("Could not parse config: {}", error))
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", &self.description)
    }
}

pub struct App {
    venv_path: std::path::PathBuf,
    requirements_lock_path: std::path::PathBuf,
    setup_py_path: std::path::PathBuf,
    python_binary: String,
}

impl App {
    pub fn new(env_name: &str) -> Result<Self, Error> {
        let current_dir = std::env::current_dir()?;
        let config_dir = appdirs::user_config_dir(None, None, false);
        // The type is Result<PathBuf, ()> I blame upstream
        if config_dir.is_err() {
            return Err(Error::new(
                "appdirs::user_data_dir() failed. That's all we know",
            ));
        }
        let config_dir = config_dir.unwrap();
        let cfg_path = config_dir.join("dmenv.toml");
        let config = std::fs::read_to_string(&cfg_path);
        if let Err(error) = config {
            return Err(Error::new(&format!(
                "Could not read from {}: {}",
                cfg_path.to_string_lossy(),
                error
            )));
        }
        let config = config.unwrap();
        let python_binary = get_python_for_env(&config, env_name)?;
        let venv_path = current_dir.join(".venv").join(env_name);
        let requirements_lock_path = current_dir.join("requirements.lock");
        let setup_py_path = current_dir.join("setup.py");
        let app = App {
            venv_path,
            requirements_lock_path,
            setup_py_path,
            python_binary,
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
        self.ensure_venv()?;

        if !self.requirements_lock_path.exists() {
            return Err(Error::new(&format!(
                "{} does not exist. Please run dmenv freeze",
                &self.requirements_lock_path.to_string_lossy(),
            )));
        }

        self.install_from_lock()
    }

    pub fn run(&self, args: Vec<String>) -> Result<(), Error> {
        let cmd = args[0].clone();
        let args: Vec<String> = args.into_iter().skip(1).collect();
        self.run_venv_cmd(&cmd, args)
    }

    pub fn freeze(&self) -> Result<(), Error> {
        self.ensure_venv()?;

        if !self.setup_py_path.exists() {
            return Err(Error::new(
                "setup.py not found. You may want to run `dmenv init` now",
            ));
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
        let args = Self::to_string_args(args);
        Self::print_cmd(&self.python_binary, &args);
        let status = std::process::Command::new(&self.python_binary)
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
        let args = Self::to_string_args(args);
        let python_str = python.to_string_lossy().to_string();
        Self::print_cmd(&python_str, &args);
        let command = std::process::Command::new(python).args(args).output()?;
        if !command.status.success() {
            return Err(Error::new(&format!(
                "pip freeze failed: {}",
                String::from_utf8_lossy(&command.stderr)
            )));
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
            "--editable",
            ".[dev]",
        ];
        self.run_venv_cmd("python", Self::to_string_args(args))
    }

    pub fn upgrade_pip(&self) -> Result<(), Error> {
        let args = vec!["-m", "pip", "install", "pip", "--upgrade"];
        self.run_venv_cmd("python", Self::to_string_args(args))
    }

    fn install_editable(&self) -> Result<(), Error> {
        // tells pip to run `setup.py develop` (that's -e), and
        // install the dev requirements too
        let args = vec!["-m", "pip", "install", "-e", ".[dev]"];
        self.run_venv_cmd("python", Self::to_string_args(args))
    }

    fn run_venv_cmd(&self, name: &str, args: Vec<String>) -> Result<(), Error> {
        let bin_path = &self.get_path_in_venv(name)?;
        Self::print_cmd(&bin_path.to_string_lossy(), &args);
        let command = std::process::Command::new(bin_path).args(args).status()?;
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

        // TODO: on Windows this is `Scripts`, not `bin`
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

    fn print_cmd(bin_path: &str, args: &Vec<String>) {
        println!(
            "{} running {} {}",
            "->".blue(),
            bin_path.bold(),
            args.join(" ")
        );
    }

    fn to_string_args(args: Vec<&str>) -> Vec<String> {
        args.iter().map(|x| x.to_string()).collect()
    }
}

#[derive(Deserialize)]
struct Config {
    env: Map<String, Env>,
}

#[derive(Deserialize)]
struct Env {
    python: String,
}

fn get_python_for_env(config: &str, env_name: &str) -> Result<String, Error> {
    let config: Config = toml::from_str(config)?;

    let matching_env = config.env.get(env_name);
    if matching_env.is_none() {
        return Err(Error::new(&format!("No such env: {}", env_name)));
    }

    let env = matching_env.unwrap();
    Ok(env.python.clone())
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_read_config() {
        let config = r#"
        [env."3.8"]
        python = "/path/to/python3.8"
        "#;

        let actual = super::get_python_for_env(&config, "3.8").unwrap();
        assert_eq!(actual, "/path/to/python3.8");

        let actual = super::get_python_for_env(&config, "nosuch");
        assert!(actual.is_err());
    }
}
