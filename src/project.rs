use std::path::PathBuf;

use crate::cmd::*;
use crate::dependencies::FrozenDependency;
use crate::error::*;
use crate::lock::BumpType;
use crate::operations;
use crate::paths::{Paths, PathsResolver};
use crate::python_info::PythonInfo;
use crate::run::VenvRunner;
use crate::settings::Settings;

#[derive(Debug)]
pub struct Metadata {
    pub dmenv_version: String,
    pub python_platform: String,
    pub python_version: String,
}

pub struct Project {
    python_info: PythonInfo,
    settings: Settings,
    paths: Paths,
    venv_runner: VenvRunner,
}

#[derive(Debug)]
pub enum PostInstallAction {
    RunSetupPyDevelop,
    None,
}

#[derive(Debug, Copy, Clone)]
pub enum ProcessScriptsMode {
    Safe,
    Override,
}

impl Project {
    pub fn new(
        project_path: PathBuf,
        python_info: PythonInfo,
        settings: Settings,
    ) -> Result<Self, Error> {
        let python_version = python_info.version.clone();
        let paths_resolver = PathsResolver::new(project_path.clone(), &python_version, &settings);
        let paths = paths_resolver.paths()?;
        let venv_runner = VenvRunner::new(&project_path, &paths.venv);
        Ok(Project {
            python_info,
            settings,
            paths,
            venv_runner,
        })
    }

    /// Allow created virtual environments to access system site packages
    pub fn use_system_site_packages(&mut self) {
        self.settings.system_site_packages = true;
    }

    /// Clean virtualenv. No-op if the virtualenv does not exist
    pub fn clean(&self) -> Result<(), Error> {
        operations::venv::clean(self.paths.venv.clone())
    }

    /// Runs `python setup.py` develop. Also called by `install` (unless InstallOptions.develop is false)
    // Note: `lock()` will use `pip install --editable .` to achieve the same effect
    pub fn develop(&self) -> Result<(), Error> {
        print_info_2("Running setup_py.py develop");
        if !self.paths.setup_py.exists() {
            return Err(Error::MissingSetupPy {});
        }

        self.venv_runner
            .run(&["python", "setup.py", "develop", "--no-deps"])
    }

    pub fn process_scripts(&self, mode: ProcessScriptsMode) -> Result<(), Error> {
        operations::scripts::process(&self.paths, mode)
    }

    /// Ensure the virtualenv exists
    //
    // Note: this is *only* called by `install()` and `lock()`.
    // All the other methods require the virtualenv to exist and
    // won't create it.
    fn ensure_venv(&self) -> Result<(), Error> {
        if self.paths.venv.exists() {
            print_info_2(&format!(
                "Using existing virtualenv: {}",
                self.paths.venv.display()
            ));
        } else {
            self.create_venv()?;
        }
        Ok(())
    }

    /// Create a new virtualenv
    //
    // Notes:
    // * The path comes from PathsResolver.paths()
    // * Called by `ensure_venv()` *if* the path does not exist
    fn create_venv(&self) -> Result<(), Error> {
        operations::venv::create(&self.paths.venv, &self.python_info, &self.settings)
    }

    /// Make sure the virtualenv exists, or return an error
    //
    // Note: this must be called by any method that requires the
    // virtualenv to exist, like `show_deps` or `run`:
    // this ensures that error messages printed when the
    // virtualenv does not exist are consistent.
    fn expect_venv(&self) -> Result<(), Error> {
        operations::venv::expect(&self.paths.venv)
    }

    /// Install dependencies from the lock file (production.lock or requirements.lock), depending
    /// on how paths were resolved by PathsResolver
    /// Abort if virtualenv or lock file does not exist
    pub fn install(&self, post_install_action: PostInstallAction) -> Result<(), Error> {
        if self.settings.production {
            print_info_1("Preparing project for production")
        } else {
            print_info_1("Preparing project for development")
        };
        let lock_path = &self.paths.lock;
        if !lock_path.exists() {
            return Err(Error::MissingLock {
                expected_path: lock_path.to_path_buf(),
            });
        }

        self.ensure_venv()?;
        self.install_from_lock()?;

        match post_install_action {
            PostInstallAction::RunSetupPyDevelop => self.develop()?,
            PostInstallAction::None => (),
        }
        Ok(())
    }

    fn install_from_lock(&self) -> Result<(), Error> {
        let lock_path = &self.paths.lock;
        print_info_2(&format!(
            "Installing dependencies from {}",
            lock_path.display()
        ));
        // Since we'll be running the command using self.paths.project
        // as working directory, we must use the *relative* lock file
        // name when calling `pip install`.
        let lock_name = &self
            .paths
            .lock
            .file_name()
            .unwrap_or_else(|| panic!("self.path.lock has no filename component"));

        let as_str = lock_name.to_string_lossy();
        let cmd = &["python", "-m", "pip", "install", "--requirement", &as_str];
        self.venv_runner.run(cmd)
    }

    pub fn upgrade_pip(&self) -> Result<(), Error> {
        print_info_2("Upgrading pip");
        let cmd = &["python", "-m", "pip", "install", "pip", "--upgrade"];
        self.venv_runner
            .run(cmd)
            .map_err(|_| Error::PipUpgradeFailed {})
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
    pub fn update_lock(&self, update_options: operations::UpdateOptions) -> Result<(), Error> {
        print_info_1("Updating lock");
        if !self.paths.setup_py.exists() {
            return Err(Error::MissingSetupPy {});
        }
        self.ensure_venv()?;
        self.upgrade_pip()?;
        self.install_editable()?;
        let metadata = &self.get_metadata()?;
        let frozen_deps = self.get_frozen_deps()?;
        let lock_path = &self.paths.lock;
        operations::lock::update(lock_path, frozen_deps, update_options, &metadata)
    }

    /// Bump a dependency in the lock file
    pub fn bump_in_lock(
        &self,
        name: &str,
        version: &str,
        bump_type: BumpType,
    ) -> Result<(), Error> {
        print_info_1(&format!("Bumping {} to {} ...", name, version));
        let metadata = self.get_metadata()?;
        operations::lock::bump(&self.paths.lock, name, version, bump_type, &metadata)
    }

    /// Run a program from the virtualenv, making sure it dies
    /// when we get killed and that the exit code is forwarded
    pub fn run_and_die<T: AsRef<str>>(&self, cmd: &[T]) -> Result<(), Error> {
        self.expect_venv()?;
        self.venv_runner.run_and_die(cmd)
    }

    /// On Windows:
    ///   - same as run
    /// On Linux:
    ///   - same as run, but create a new process instead of using execv()
    // Note: mostly for tests. We want to *check* the return code of
    // `dmenv run` and so we need a child process
    pub fn run<T: AsRef<str>>(&self, cmd: &[T]) -> Result<(), Error> {
        self.expect_venv()?;
        self.venv_runner.run(cmd)
    }

    /// Show the dependencies inside the virtualenv.
    // Note: Run `pip list` so we get what's *actually* installed, not just
    // the contents of the lock file
    pub fn show_deps(&self) -> Result<(), Error> {
        self.venv_runner.run(&["python", "-m", "pip", "list"])
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
        self.expect_venv()?;
        let bin_path = self.venv_runner.binaries_path();
        println!("{}", bin_path.display());
        Ok(())
    }

    pub fn show_outdated(&self) -> Result<(), Error> {
        #[rustfmt::skip]
        let cmd = &[
            "python", "-m", "pip",
            "list", "--outdated",
            "--format", "columns",
        ];
        self.venv_runner.run(cmd)
    }

    fn install_editable(&self) -> Result<(), Error> {
        let mut message = "Installing deps from setup.py".to_string();
        if self.settings.production {
            message.push_str(" using 'prod' extra dependencies");
        } else {
            message.push_str(" using 'dev' extra dependencies");
        }
        print_info_2(&message);

        let extra = if self.settings.production {
            ".[prod]"
        } else {
            ".[dev]"
        };
        let cmd = &["python", "-m", "pip", "install", "--editable", extra];
        self.venv_runner.run(cmd)
    }

    fn get_metadata(&self) -> Result<Metadata, Error> {
        let dmenv_version = env!("CARGO_PKG_VERSION");
        let python_platform = &self.python_info.platform;
        let python_version = &self.python_info.version;
        Ok(Metadata {
            dmenv_version: dmenv_version.to_string(),
            python_platform: python_platform.to_string(),
            python_version: python_version.to_string(),
        })
    }

    /// Get the list of the *actual* deps in the virtualenv by calling `pip freeze`.
    fn get_frozen_deps(&self) -> Result<Vec<FrozenDependency>, Error> {
        let freeze_output = self.run_pip_freeze()?;
        // First, collect all the `pip freeze` lines into frozen dependencies
        let deps: Result<Vec<_>, _> = freeze_output
            .lines()
            .map(|x| FrozenDependency::from_string(x.into()))
            .collect();
        let deps = deps?;
        // Then filter out pkg-resources: this works around a Debian bug in pip:
        // https://bugs.debian.org/cgi-bin/bugreport.cgi?bug=871790
        let res: Vec<_> = deps
            .into_iter()
            .filter(|x| x.name != "pkg-resources")
            .collect();
        Ok(res)
    }

    fn run_pip_freeze(&self) -> Result<String, Error> {
        let lock_path = &self.paths.lock;
        print_info_2(&format!("Generating {}", lock_path.display()));
        #[rustfmt::skip]
        let cmd = &[
            "python", "-m", "pip", "freeze",
            "--exclude-editable",
            "--all",
            "--local",
        ];
        self.venv_runner.get_output(cmd)
    }
}
