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
}
