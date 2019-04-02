use colored::*;
use std::path::PathBuf;

#[cfg(unix)]
use crate::execv::execv;
#[cfg(windows)]
use crate::win_job;

use crate::cmd::*;
use crate::dependencies::FrozenDependency;
use crate::error::*;
use crate::lock::Lock;
use crate::paths::Paths;
use crate::python_info::PythonInfo;
use crate::settings::Settings;

struct LockMetadata {
    dmenv_version: String,
    python_platform: String,
    python_version: String,
}

#[derive(Default)]
/// Represents options passed to `dmenv lock`,
/// see `cmd::SubCommand::Lock`
pub struct LockOptions {
    pub python_version: Option<String>,
    pub sys_platform: Option<String>,
    pub system_site_packages: bool,
}

#[derive(Default)]
/// Represents options passed to `dmenv install`
/// see `cmd::SubCommand::Install`
pub struct InstallOptions {
    pub develop: bool,
    pub system_site_packages: bool,
}

pub struct VenvManager {
    paths: Paths,
    python_info: PythonInfo,
    settings: Settings,
}

impl VenvManager {
    pub fn new(paths: Paths, python_info: PythonInfo, settings: Settings) -> Self {
        VenvManager {
            paths,
            settings,
            python_info,
        }
    }

    /// Clean virtualenv. No-op if the virtualenv does not exist
    pub fn clean(&self) -> Result<(), Error> {
        print_info_1(&format!("Cleaning {}", &self.paths.venv.display()));
        if !self.paths.venv.exists() {
            return Ok(());
        }
        std::fs::remove_dir_all(&self.paths.venv).map_err(|e| Error::Other {
            message: format!("could not remove {}: {}", &self.paths.venv.display(), e),
        })
    }

    /// Runs `python setup.py` develop. Also called by `install` (unless InstallOptions.develop is false)
    // Note: `lock()` will use `pip install --editable .` to achieve the same effect
    pub fn develop(&self) -> Result<(), Error> {
        print_info_2("Running setup_py.py develop");
        if !self.paths.setup_py.exists() {
            return Err(Error::MissingSetupPy {});
        }

        self.run_cmd_in_venv("python", vec!["setup.py", "develop", "--no-deps"])
    }

    /// Install dependencies from lock file (production.lock or requirements.lock), depending
    /// on how paths were resolved by PathsResolver
    /// Abort if virtualenv or lock file does not exist
    pub fn install(&self, install_options: &InstallOptions) -> Result<(), Error> {
        print_info_1("Preparing project for development");
        let lock_path = &self.paths.lock;
        if !lock_path.exists() {
            return Err(Error::MissingLock {
                expected_path: lock_path.to_path_buf(),
            });
        }

        self.ensure_venv(install_options.system_site_packages)?;
        self.install_from_lock()?;

        if install_options.develop {
            self.develop()?;
        }
        Ok(())
    }

    /// Run a program from the virtualenv, making sure it dies
    /// when we get killed and that the exit code is forwarded
    pub fn run(&self, args: &[String]) -> Result<(), Error> {
        #[cfg(windows)]
        {
            unsafe {
                win_job::setup();
            }
            self.run_no_exec(args)
        }

        #[cfg(unix)]
        {
            let bin_path = &self.get_path_in_venv(&args[0])?;
            let bin_path_str = bin_path.to_str().ok_or(Error::Other {
                message: "Could not convert binary path to String".to_string(),
            })?;
            let mut fixed_args: Vec<String> = args.to_vec();
            fixed_args[0] = bin_path_str.to_string();
            execv(bin_path_str, fixed_args)
        }
    }

    /// On Windows:
    ///   - same as run
    /// On Linux:
    ///   - same as run, but create a new process instead of using execv()
    // Note: mostly for tests. We want to *check* the return code of
    // `dmenv run` and so we need a child process
    pub fn run_no_exec(&self, args: &[String]) -> Result<(), Error> {
        self.expect_venv()?;
        let cmd = args[0].clone();
        let args: Vec<&str> = args.iter().skip(1).map(String::as_str).collect();
        self.run_cmd_in_venv(&cmd, args)
    }

    /// (Re)generate the lock file
    //
    // Notes:
    //
    // * Abort if `setup.py` is not found
    // * Create the virtualenv if required
    // * Always upgrade pip :
    //    * If that fails, we know if the virtualenv is broken
    //    * Also, we know sure that `pip` can handle all the options
    //      (such as `--local`, `--exclude-editable`) we use in the other functions
    // * The path of the lock file is computed by PathsResolver.
    //     See PathsResolver.paths() for details
    //
    // * Delegates the actual work to `write_lock()`
    //
    pub fn lock(&self, lock_options: &LockOptions) -> Result<(), Error> {
        print_info_1("Locking dependencies");
        if !self.paths.setup_py.exists() {
            return Err(Error::MissingSetupPy {});
        }

        self.ensure_venv(lock_options.system_site_packages)?;
        self.upgrade_pip()?;

        self.install_editable()?;

        self.write_lock(&lock_options)?;
        Ok(())
    }

    /// Show the dependencies inside the virtualenv.
    // Note: Run `pip list` so we get what's *actually* installed, not just
    // the contents of the lock file
    pub fn show_deps(&self) -> Result<(), Error> {
        self.run_cmd_in_venv("pip", vec!["list"])
    }

    /// Show the resolved virtualenv path.
    //
    // See `PathsResolver.paths()` for details
    pub fn show_venv_path(&self) -> Result<(), Error> {
        println!("{}", self.paths.venv.display());
        Ok(())
    }

    /// Same has `show_venv_path`, but add the correct subfolder
    /// (`bin` on Linux and macOS, `Scripts` on Windows).
    pub fn show_venv_bin_path(&self) -> Result<(), Error> {
        let bin_path = &self.get_venv_bin_path();
        println!("{}", bin_path.display());
        Ok(())
    }

    pub fn show_outdated(&self) -> Result<(), Error> {
        self.run_cmd_in_venv("pip", vec!["list", "--outdated", "--format", "columns"])
    }

    /// Creates `setup.py` if it does not exist.
    pub fn init(&self, name: &str, version: &str, author: &Option<String>) -> Result<(), Error> {
        let path = &self.paths.setup_py;
        if path.exists() {
            return Err(Error::FileExists {
                path: path.to_path_buf(),
            });
        }
        // Warning: make sure the source file in `src/setup.in.py` contains all those
        // placeholders
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

    /// Bump a dependency in the lock file
    //
    // Note: most of the work is delegated to the Lock struct. Either `Lock.git_bump()`or
    // `Lock.bump()` is called, depending on the value of the `git` argument.
    pub fn bump_in_lock(&self, name: &str, version: &str, git: bool) -> Result<(), Error> {
        print_info_1(&format!("Bumping {} to {} ...", name, version));
        let path = &self.paths.lock;
        let lock_contents = std::fs::read_to_string(&path).map_err(|e| Error::ReadError {
            path: path.to_path_buf(),
            io_error: e,
        })?;
        let mut lock = Lock::from_string(&lock_contents)?;
        let changed = if git {
            lock.git_bump(name, version)
        } else {
            lock.bump(name, version)
        }?;
        if !changed {
            print_warning(&format!("Dependency {} already up-to-date", name.bold()));
            return Ok(());
        }
        let new_contents = lock.to_string();
        std::fs::write(&path, &new_contents).map_err(|e| Error::WriteError {
            path: path.to_path_buf(),
            io_error: e,
        })?;
        println!("{}", "ok!".green());
        Ok(())
    }

    /// Ensure the virtualenv exists
    //
    // Note: this is *only* called by `install()` and `lock()`.
    // All the other methods require the virtualenv to exist and
    // won't create it.
    fn ensure_venv(&self, system_site_packages: bool) -> Result<(), Error> {
        if self.paths.venv.exists() {
            print_info_2(&format!(
                "Using existing virtualenv: {}",
                self.paths.venv.display()
            ));
        } else {
            self.create_venv(system_site_packages)?;
        }
        Ok(())
    }

    /// Make sure the virtualenv exists, or return an error
    //
    // Note: this must be called by any method that requires the
    // virtualenv to exist, like `show_deps` or `run`:
    // this ensures that error messages printed when the
    // virtualenv does not exist are consistent.
    fn expect_venv(&self) -> Result<(), Error> {
        if !self.paths.venv.exists() {
            return Err(Error::MissingVenv {
                path: self.paths.venv.clone(),
            });
        }
        Ok(())
    }

    /// Create a new virtualenv
    //
    // Notes:
    // * The path comes from PathsResolver.paths()
    // * Called by `ensure_venv()` *if* the path does not exist
    fn create_venv(&self, system_site_packages: bool) -> Result<(), Error> {
        let parent_venv_path = &self.paths.venv.parent().ok_or(Error::Other {
            message: "venv_path has no parent".to_string(),
        })?;
        print_info_2(&format!(
            "Creating virtualenv in: {}",
            self.paths.venv.display()
        ));
        std::fs::create_dir_all(&parent_venv_path).map_err(|e| Error::Other {
            message: format!("Could not create {}: {}", parent_venv_path.display(), e),
        })?;

        // Python -m venv should work in most cases (venv is in the stdlib since Python 3.3)
        let venv_path = &self.paths.venv.to_string_lossy();
        let mut args = vec!["-m"];
        if self.settings.venv_from_stdlib {
            args.push("venv")
        } else {
            // In case we can't or won't use venv from the stdlib, use `virtualenv` instead.
            // Assume the virtualenv package is present on the system.
            args.push("virtualenv")
        };
        args.push(venv_path);
        if system_site_packages {
            args.push("--system-site-packages");
        }
        let python_binary = &self.python_info.binary;
        Self::print_cmd(&python_binary.to_string_lossy(), &args);
        let status = std::process::Command::new(&python_binary)
            .current_dir(&self.paths.project)
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

    // Actually write the lock file
    // Delegates most of the work to the Lock struct.
    fn write_lock(&self, lock_options: &LockOptions) -> Result<(), Error> {
        let metadata = &self.get_metadata()?;

        let lock_path = &self.paths.lock;
        let lock_contents = if lock_path.exists() {
            std::fs::read_to_string(&lock_path).map_err(|e| Error::ReadError {
                path: lock_path.to_owned(),
                io_error: e,
            })?
        } else {
            String::new()
        };

        let mut lock = Lock::from_string(&lock_contents)?;
        if let Some(python_version) = &lock_options.python_version {
            lock.python_version(&python_version);
        }
        if let Some(sys_platform) = &lock_options.sys_platform {
            lock.sys_platform(&sys_platform);
        }
        let frozen_deps = self.get_frozen_deps()?;
        lock.freeze(&frozen_deps);
        let new_contents = lock.to_string();

        let LockMetadata {
            dmenv_version,
            python_version,
            python_platform,
        } = metadata;
        let top_comment = format!(
            "# Generated with dmenv {}, python {}, on {}\n",
            dmenv_version, &python_version, &python_platform
        );

        let to_write = top_comment + &new_contents;
        std::fs::write(&lock_path, &to_write).map_err(|e| Error::WriteError {
            path: lock_path.to_path_buf(),
            io_error: e,
        })
    }

    /// Get the list of the *actual* deps in the virtualenv by calling `pip freeze`.
    fn get_frozen_deps(&self) -> Result<Vec<FrozenDependency>, Error> {
        let freeze_output = self.run_pip_freeze()?;
        let mut res = vec![];
        for line in freeze_output.lines() {
            let frozen_dep = FrozenDependency::from_string(&line)?;
            // Filter out pkg-resources. This works around
            // a Debian bug in pip: https://bugs.debian.org/cgi-bin/bugreport.cgi?bug=871790
            if frozen_dep.name != "pkg-resources" {
                res.push(frozen_dep);
            }
        }

        Ok(res)
    }

    fn run_pip_freeze(&self) -> Result<String, Error> {
        let lock_path = &self.paths.lock;
        print_info_2(&format!("Generating {}", lock_path.display()));
        let pip = self.get_path_in_venv("pip")?;
        let pip_str = pip.to_string_lossy().to_string();
        let args = vec!["freeze", "--exclude-editable", "--all", "--local"];
        Self::print_cmd(&pip_str, &args);
        let command = std::process::Command::new(pip)
            .current_dir(&self.paths.project)
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
        Ok(String::from_utf8_lossy(&command.stdout).to_string())
    }

    fn get_metadata(&self) -> Result<LockMetadata, Error> {
        let dmenv_version = env!("CARGO_PKG_VERSION");
        let python_platform = &self.python_info.platform;
        let python_version = &self.python_info.version;
        Ok(LockMetadata {
            dmenv_version: dmenv_version.to_string(),
            python_platform: python_platform.to_string(),
            python_version: python_version.to_string(),
        })
    }

    fn install_from_lock(&self) -> Result<(), Error> {
        let lock_path = &self.paths.lock;
        print_info_2(&format!(
            "Installing dependencies from {}",
            lock_path.display()
        ));
        let as_str = &self.paths.lock.to_string_lossy();
        let args = vec!["-m", "pip", "install", "--requirement", as_str];
        self.run_cmd_in_venv("python", args)
    }

    pub fn upgrade_pip(&self) -> Result<(), Error> {
        print_info_2("Upgrading pip");
        let args = vec!["-m", "pip", "install", "pip", "--upgrade"];
        self.run_cmd_in_venv("python", args)
            .map_err(|_| Error::PipUpgradeFailed {})
    }

    fn install_editable(&self) -> Result<(), Error> {
        let mut message = "Installing deps from setup.py".to_string();
        if self.settings.production {
            message.push_str(" using 'prod' extra dependencies");
        } else {
            message.push_str(" using 'dev' extra dependencies");
        }
        print_info_2(&message);

        let mut args = vec!["-m", "pip", "install", "--editable"];
        if self.settings.production {
            args.push(".[prod]")
        } else {
            args.push(".[dev]")
        }
        self.run_cmd_in_venv("python", args)
    }

    fn run_cmd_in_venv(&self, name: &str, args: Vec<&str>) -> Result<(), Error> {
        let bin_path = &self.get_path_in_venv(name)?;
        Self::print_cmd(&bin_path.to_string_lossy(), &args);
        let command = std::process::Command::new(bin_path)
            .args(args)
            .current_dir(&self.paths.project)
            .status();
        let command = command.map_err(|e| Error::ProcessWaitError { io_error: e })?;
        if !command.success() {
            return Err(Error::Other {
                message: "command failed".to_string(),
            });
        }

        Ok(())
    }

    fn get_venv_bin_path(&self) -> PathBuf {
        #[cfg(not(windows))]
        let binaries_subdirs = "bin";

        #[cfg(windows)]
        let binaries_subdirs = "Scripts";

        self.paths.venv.join(binaries_subdirs)
    }

    fn get_path_in_venv(&self, name: &str) -> Result<PathBuf, Error> {
        if !self.paths.venv.exists() {
            return Err(Error::Other {
                message: format!(
                    "virtualenv in {} does not exist",
                    &self.paths.venv.display()
                ),
            });
        }

        #[cfg(windows)]
        let suffix = ".exe";
        #[cfg(not(windows))]
        let suffix = "";

        let name = format!("{}{}", name, suffix);
        let bin_path = &self.get_venv_bin_path();
        let path = self.paths.venv.join(bin_path).join(name);
        if !path.exists() {
            return Err(Error::Other {
                message: format!("Cannot run: '{}' does not exist", &path.display()),
            });
        }
        Ok(path)
    }

    fn print_cmd(bin_path: &str, args: &[&str]) {
        println!("{} {} {}", "$".blue(), bin_path, args.join(" "));
    }
}
