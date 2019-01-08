use crate::cmd::*;
use colored::*;

use crate::error::*;
use crate::lock::Lock;
use crate::python_info::PythonInfo;
use std::ffi::CString;
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
        std::fs::remove_dir_all(&self.paths.venv).map_err(|e| Error::Other {
            message: format!(
                "could not remove {}: {}",
                &self.paths.venv.to_string_lossy(),
                e
            ),
        })
    }

    pub fn develop(&self) -> Result<(), Error> {
        if !self.paths.setup_py.exists() {
            return Err(Error::MissingSetupPy {});
        }

        self.run_cmd_in_venv("python", vec!["setup.py", "develop", "--no-deps"])
    }

    pub fn install(&self, install_options: &InstallOptions) -> Result<(), Error> {
        if !self.paths.lock.exists() {
            return Err(Error::MissingLock {});
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
        self.expect_venv()?;
        let bin_path = &self.get_path_in_venv(&args[0])?;
        let bin_path_str = bin_path.to_str().ok_or(Error::Other {
            message: "Could not convert binary path to char*".to_string(),
        })?;
        let mut fixed_args: Vec<String> = args.to_vec();
        fixed_args[0] = bin_path_str.to_string();

        let mut args_cstring = vec![];
        for arg in fixed_args {
            args_cstring.push(to_c_string(&arg)?);
        }
        let mut args_ptr: Vec<_> = args_cstring.iter().map(|x| x.as_ptr()).collect();
        args_ptr.push(std::ptr::null());
        let c_program = to_c_string(bin_path_str)?;
        let rc;
        unsafe {
            rc = libc::execv(c_program.as_ptr(), args_ptr.as_ptr());
        }
        if rc != 0 {
            let error = std::io::Error::last_os_error();
            return Err(Error::ProcessStartError {
                message: format!("execv() failed: {}", error),
            });
        }

        Ok(())
    }

    /// Same as `run()`, but use `execv()` instead of
    /// forking a new process.
    pub fn run_no_exec(&self, args: &[String]) -> Result<(), Error> {
        self.expect_venv()?;
        let cmd = args[0].clone();
        let args: Vec<&str> = args.iter().skip(1).map(|x| x.as_str()).collect();
        self.run_cmd_in_venv(&cmd, args)
    }

    pub fn lock(&self) -> Result<(), Error> {
        if !self.paths.setup_py.exists() {
            return Err(Error::MissingSetupPy {});
        }

        self.write_metadata()?;
        self.ensure_venv()?;
        self.upgrade_pip()?;

        print_info_1("Generating requirements.lock from setup.py");
        self.install_editable()?;
        self.run_pip_freeze()?;
        Ok(())
    }

    pub fn show_deps(&self) -> Result<(), Error> {
        self.run_cmd_in_venv("pip", vec!["list"])
    }

    pub fn show_venv_path(&self) -> Result<(), Error> {
        println!("{}", self.paths.venv.to_string_lossy());
        Ok(())
    }

    pub fn init(&self, name: &str, version: &str, author: &Option<String>) -> Result<(), Error> {
        let path = &self.paths.setup_py;
        if path.exists() {
            return Err(Error::FileExists {
                path: path.to_path_buf(),
            });
        }
        let template = include_str!("setup.in.py");
        let with_name = template.replace("<NAME>", name);
        let with_version = with_name.replace("<VERSION>", version);
        let to_write = if let Some(author) = author {
            with_version.replace("<AUTHOR>", author)
        } else {
            with_version
        };
        std::fs::write(&path, to_write).map_err(|e| Error::WriteError {
            path: path.to_path_buf(),
            io_error: e,
        })?;
        print_info_1("Generated a new setup.py");
        Ok(())
    }

    pub fn bump_in_lock(&self, name: &str, version: &str, git: bool) -> Result<(), Error> {
        let path = &self.paths.lock;
        let lock_contents = std::fs::read_to_string(&path).map_err(|e| Error::ReadError {
            path: path.to_path_buf(),
            io_error: e,
        })?;
        let lock = Lock::new(&lock_contents);
        let new_contents = if git {
            lock.git_bump(name, version)
        } else {
            lock.bump(name, version)
        }?;
        let path = &self.paths.lock;
        std::fs::write(&path, &new_contents).map_err(|e| Error::WriteError {
            path: path.to_path_buf(),
            io_error: e,
        })?;
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

    fn expect_venv(&self) -> Result<(), Error> {
        if !self.paths.venv.exists() {
            return Err(Error::MissingVenv {
                path: self.paths.venv.clone(),
            });
        }
        Ok(())
    }

    fn create_venv(&self) -> Result<(), Error> {
        let parent_venv_path = &self.paths.venv.parent().ok_or(Error::Other {
            message: "venv_path has no parent".to_string(),
        })?;
        print_info_1(&format!(
            "Creating virtualenv in: {}",
            self.paths.venv.to_string_lossy()
        ));
        std::fs::create_dir_all(&parent_venv_path).map_err(|e| Error::Other {
            message: format!(
                "Could not create {}: {}",
                parent_venv_path.to_string_lossy(),
                e
            ),
        })?;
        let venv_path = &self.paths.venv.to_string_lossy();
        let mut args = vec!["-m"];
        if self.python_info.venv_from_stdlib {
            args.push("venv")
        } else {
            args.push("virtualenv")
        };
        args.push(venv_path);
        let python_binary = &self.python_info.binary;
        Self::print_cmd(&python_binary.to_string_lossy(), &args);
        let status = std::process::Command::new(&python_binary)
            .current_dir(&self.paths.working_dir)
            .args(&args)
            .status();
        let status = status.map_err(|e| Error::ProcessWaitError { io_error: e })?;
        if !status.success() {
            return Err(Error::Other {
                message: "failed to create virtualenv".to_string(),
            });
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
        let command = command.map_err(|e| Error::ProcessOutError { io_error: e })?;
        if !command.status.success() {
            return Err(Error::Other {
                message: format!(
                    "pip freeze failed: {}",
                    String::from_utf8_lossy(&command.stderr)
                ),
            });
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
        let mut file = file.map_err(|e| Error::WriteError {
            path: path.to_path_buf(),
            io_error: e,
        })?;
        let mut to_write = lines.join("\n");
        to_write.push_str("\n");
        file.write_all(to_write.as_bytes())
            .map_err(|e| Error::WriteError {
                path: path.to_path_buf(),
                io_error: e,
            })?;
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
        std::fs::write(&path, &comment).map_err(|e| Error::WriteError {
            path: path.to_path_buf(),
            io_error: e,
        })?;
        Ok(())
    }

    fn install_from_lock(&self) -> Result<(), Error> {
        let as_str = &self.paths.lock.to_string_lossy();
        let args = vec!["install", "--requirement", as_str];
        self.run_cmd_in_venv("pip", args)
    }

    pub fn upgrade_pip(&self) -> Result<(), Error> {
        let args = vec!["-m", "pip", "install", "pip", "--upgrade"];
        self.run_cmd_in_venv("python", args)
    }

    fn install_editable(&self) -> Result<(), Error> {
        // tells pip to run `setup.py develop` (that's -e), and
        // install the dev requirements too
        let args = vec!["-m", "pip", "install", "-e", ".[dev]"];
        self.run_cmd_in_venv("python", args)
    }

    fn run_cmd_in_venv(&self, name: &str, args: Vec<&str>) -> Result<(), Error> {
        let bin_path = &self.get_path_in_venv(name)?;
        Self::print_cmd(&bin_path.to_string_lossy(), &args);
        let command = std::process::Command::new(bin_path)
            .args(args)
            .current_dir(&self.paths.working_dir)
            .status();
        let command = command.map_err(|e| Error::ProcessWaitError { io_error: e })?;
        if !command.success() {
            return Err(Error::Other {
                message: "command failed".to_string(),
            });
        }

        Ok(())
    }

    fn get_path_in_venv(&self, name: &str) -> Result<std::path::PathBuf, Error> {
        if !self.paths.venv.exists() {
            return Err(Error::Other {
                message: format!(
                    "virtualenv in {} does not exist",
                    &self.paths.venv.to_string_lossy()
                ),
            });
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
            return Err(Error::Other {
                message: format!("Cannot run: '{}' does not exist", &path.to_string_lossy()),
            });
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

fn to_c_string(string: &str) -> Result<CString, Error> {
    CString::new(string.to_string()).map_err(|_| Error::NulByteFound {
        arg: string.to_string(),
    })
}
