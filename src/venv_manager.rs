extern crate colored;
use cmd::*;
use colored::*;

use error;
use error::*;
use lock::Lock;
use python_info::PythonInfo;
use std::io::Write;

pub const LOCK_FILE_NAME: &str = "requirements.lock";

pub struct VenvManager {
    paths: Paths,
    python_info: PythonInfo,
}

#[derive(Default)]
pub struct InstallOptions {
    pub develop: bool,
    pub upgrade_pip: bool,
}

impl VenvManager {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(working_dir: std::path::PathBuf, python_info: PythonInfo) -> Result<Self, Error> {
        let lock_path = working_dir.join(LOCK_FILE_NAME);
        let setup_py_path = working_dir.join("setup.py");
        let venv_path = if let Ok(env_var) = std::env::var("VIRTUAL_ENV") {
            std::path::PathBuf::from(env_var)
        } else {
            working_dir.join(".venv").join(&python_info.version)
        };
        let paths = Paths {
            working_dir,
            venv: venv_path,
            lock: lock_path,
            setup_py: setup_py_path,
        };
        let venv_manager = VenvManager { paths, python_info };
        Ok(venv_manager)
    }

    pub fn clean(&self) -> Result<(), Error> {
        print_info_1(&format!("Cleaning {}", &self.paths.venv.to_string_lossy()));
        if !self.paths.venv.exists() {
            return Ok(());
        }
        let outcome = std::fs::remove_dir_all(&self.paths.venv);
        match outcome {
            Ok(()) => Ok(()),
            Err(e) => error::new(&format!(
                "could not remove {}: {}",
                &self.paths.venv.to_string_lossy(),
                e
            )),
        }
    }

    pub fn develop(&self) -> Result<(), Error> {
        if !self.paths.setup_py.exists() {
            return error::missing_setup_py();
        }

        self.run_venv_cmd("python", vec!["setup.py", "develop", "--no-deps"])
    }

    pub fn install(&self, install_options: &InstallOptions) -> Result<(), Error> {
        if !self.paths.lock.exists() {
            return error::missing_lock();
        }

        self.ensure_venv()?;
        if install_options.upgrade_pip {
            self.upgrade_pip()?;
        }
        self.install_from_lock()?;

        if install_options.develop {
            self.develop()?;
        }
        Ok(())
    }

    pub fn run(&self, args: &[String]) -> Result<(), Error> {
        if !self.paths.venv.exists() {
            return error::missing_venv(self.paths.venv.clone());
        }
        let cmd = args[0].clone();
        let args: Vec<&str> = args.iter().skip(1).map(|x| x.as_str()).collect();
        self.run_venv_cmd(&cmd, args)
    }

    pub fn lock(&self) -> Result<(), Error> {
        if !self.paths.setup_py.exists() {
            return error::missing_setup_py();
        }

        self.write_metadata()?;
        self.ensure_venv()?;
        self.upgrade_pip()?;

        print_info_1("Generating requirements.txt from setup.py");
        self.install_editable()?;
        self.run_pip_freeze()?;
        Ok(())
    }

    pub fn show_deps(&self) -> Result<(), Error> {
        self.run_venv_cmd("pip", vec!["list"])
    }

    pub fn show_venv_path(&self) -> Result<(), Error> {
        println!("{}", self.paths.venv.to_string_lossy());
        Ok(())
    }

    pub fn init(&self, name: &str, version: &str, author: &Option<String>) -> Result<(), Error> {
        let path = &self.paths.setup_py;
        if path.exists() {
            return error::file_exists(path.clone());
        }
        let template = include_str!("setup.in.py");
        let with_name = template.replace("<NAME>", name);
        let with_version = with_name.replace("<VERSION>", version);
        let to_write = if let Some(author) = author {
            with_version.replace("<AUTHOR>", author)
        } else {
            with_version
        };
        let res = std::fs::write(&path, to_write);
        if let Err(e) = res {
            return error::write(path.clone(), e);
        }
        print_info_1("Generated a new setup.py");
        Ok(())
    }

    pub fn bump_in_lock(&self, name: &str, version: &str, git: bool) -> Result<(), Error> {
        let path = &self.paths.lock;
        let lock_contents = std::fs::read_to_string(&path);
        if let Err(e) = lock_contents {
            return error::read(path.clone(), e);
        }
        let lock_contents = lock_contents.unwrap();
        let lock = Lock::new(&lock_contents);
        let new_contents = if git {
            lock.git_bump(name, version)
        } else {
            lock.bump(name, version)
        }?;
        let path = &self.paths.lock;
        let res = std::fs::write(&path, &new_contents);
        if let Err(e) = res {
            return error::write(path.clone(), e);
        }
        Ok(())
    }

    fn ensure_venv(&self) -> Result<(), Error> {
        if self.paths.venv.exists() {
            print_info_1(&format!(
                "Using existing virtualenv: {}",
                self.paths.venv.to_string_lossy()
            ));
        } else {
            self.create_venv()?;
        }
        Ok(())
    }

    fn create_venv(&self) -> Result<(), Error> {
        let parent_venv_path = &self.paths.venv.parent();
        if parent_venv_path.is_none() {
            return error::new("venv_path has no parent");
        }
        let parent_venv_path = parent_venv_path.unwrap();
        print_info_1(&format!(
            "Creating virtualenv in: {}",
            self.paths.venv.to_string_lossy()
        ));
        let res = std::fs::create_dir_all(&parent_venv_path);
        if let Err(e) = res {
            return error::new(&format!(
                "Could not create {}: {}",
                parent_venv_path.to_string_lossy(),
                e
            ));
        }
        let venv_path = &self.paths.venv.to_string_lossy();
        let args = vec!["-m", "venv", venv_path];
        let python_binary = &self.python_info.binary;
        Self::print_cmd(&python_binary.to_string_lossy(), &args);
        let status = std::process::Command::new(&python_binary)
            .current_dir(&self.paths.working_dir)
            .args(&args)
            .status();
        if let Err(e) = status {
            return error::process_wait(e);
        }
        let status = status.unwrap();
        if !status.success() {
            return error::new("failed to create virtualenv");
        }
        Ok(())
    }

    fn run_pip_freeze(&self) -> Result<(), Error> {
        let pip = self.get_path_in_venv("pip")?;
        let pip_str = pip.to_string_lossy().to_string();
        let args = vec!["freeze", "--exclude-editable"];
        Self::print_cmd(&pip_str, &args);
        let command = std::process::Command::new(pip)
            .current_dir(&self.paths.working_dir)
            .args(args)
            .output();
        if let Err(e) = command {
            return error::process_out(e);
        }
        let command = command.unwrap();
        if !command.status.success() {
            return error::new(&format!(
                "pip freeze failed: {}",
                String::from_utf8_lossy(&command.stderr)
            ));
        }
        let out = String::from_utf8_lossy(&command.stdout);

        // Filter out pkg-resources. This works around
        // a Debian bug in pip: https://bugs.debian.org/cgi-bin/bugreport.cgi?bug=871790
        let mut lines = vec![];
        for line in out.lines() {
            if !line.starts_with("pkg-resources==") {
                lines.push(line);
            }
        }
        let path = &self.paths.lock;
        let file = std::fs::OpenOptions::new().append(true).open(&path);
        if let Err(e) = file {
            return error::write(path.clone(), e);
        }
        let mut file = file.unwrap();
        let mut to_write = lines.join("\n");
        to_write.push_str("\n");
        let res = file.write_all(to_write.as_bytes());
        if let Err(e) = res {
            return error::write(path.clone(), e);
        }
        println!(
            "{} Requirements written to {}",
            "::".blue(),
            self.paths.lock.to_string_lossy()
        );
        Ok(())
    }

    fn write_metadata(&self) -> Result<(), Error> {
        let dmenv_version = env!("CARGO_PKG_VERSION");
        let python_platform = &self.python_info.platform;
        let python_version = &self.python_info.version;
        let comment = format!(
            "# Generated with dmenv {}, python {}, on {}\n",
            dmenv_version, &python_version, &python_platform
        );
        let path = &self.paths.lock;
        let res = std::fs::write(&path, &comment);
        if let Err(e) = res {
            return error::write(path.clone(), e);
        }
        Ok(())
    }

    fn install_from_lock(&self) -> Result<(), Error> {
        let as_str = &self.paths.lock.to_string_lossy();
        let args = vec!["install", "--requirement", as_str];
        self.run_venv_cmd("pip", args)
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
            .current_dir(&self.paths.working_dir)
            .status();
        if let Err(e) = command {
            return error::process_wait(e);
        }
        let command = command.unwrap();
        if !command.success() {
            return error::new("command failed");
        }

        Ok(())
    }

    fn get_path_in_venv(&self, name: &str) -> Result<std::path::PathBuf, Error> {
        if !self.paths.venv.exists() {
            return error::new(&format!(
                "virtualenv in {} does not exist",
                &self.paths.venv.to_string_lossy()
            ));
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
        let path = self.paths.venv.join(binaries_subdirs).join(name);
        if !path.exists() {
            return error::new(&format!(
                "Cannot run: '{}' does not exist",
                &path.to_string_lossy()
            ));
        }
        Ok(path)
    }

    fn print_cmd(bin_path: &str, args: &[&str]) {
        print_info_2(&format!("Running {} {}", bin_path.bold(), args.join(" ")));
    }
}

struct Paths {
    working_dir: std::path::PathBuf,
    venv: std::path::PathBuf,
    lock: std::path::PathBuf,
    setup_py: std::path::PathBuf,
}
