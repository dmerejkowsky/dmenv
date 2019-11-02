use std::path::PathBuf;

use crate::cmd::*;
use crate::dependencies::FrozenDependency;
use crate::error::*;
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

#[derive(Debug)]
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
        let paths_resolver = PathsResolver::new(project_path.clone(), python_version, &settings);
        let paths = paths_resolver.paths()?;
        let venv_runner = VenvRunner::new(&project_path, &paths.venv);
        Ok(Project {
            python_info,
            settings,
            paths,
            venv_runner,
        })
    }

    /// Clean virtualenv. No-op if the virtualenv does not exist
    pub fn clean_venv(&self) -> Result<(), Error> {
        operations::venv::clean(self.paths.venv.clone())
    }

    /// Create a new virtualenv
    //
    // Notes:
    // * The path comes from PathsResolver.paths()
    // * Called by `ensure_venv()` *if* the path does not exist
    fn create_venv(&self) -> Result<(), Error> {
        operations::venv::create(&self.paths.venv, &self.python_info, &self.settings)
    }

    fn get_install_editable_cmd(&self) -> [&str; 6] {
        let extra = if self.settings.production {
            ".[prod]"
        } else {
            ".[dev]"
        };
        ["python", "-m", "pip", "install", "--editable", extra]
    }

    fn metadata(&self) -> Metadata {
        let dmenv_version = env!("CARGO_PKG_VERSION");
        let python_platform = &self.python_info.platform;
        let python_version = &self.python_info.version;
        Metadata {
            dmenv_version: dmenv_version.to_string(),
            python_platform: python_platform.to_string(),
            python_version: python_version.to_string(),
        }
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
