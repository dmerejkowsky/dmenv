use crate::settings::Settings;
use app_dirs::{AppDataType, AppInfo};
use std::path::PathBuf;

const APP_INFO: AppInfo = AppInfo {
    name: "dmenv",
    author: "Tanker",
};

pub const PROD_LOCK_FILENAME: &str = "production.lock";
pub const DEV_LOCK_FILENAME: &str = "requirements.lock";

use crate::error::*;

// Container for all the PathsBuf used by the venv_manager
pub struct Paths {
    pub project: PathBuf,
    pub venv: PathBuf,
    pub lock: PathBuf,
    pub setup_py: PathBuf,
}

pub struct PathsResolver {
    venv_outside_project: bool,
    production: bool,
    python_version: String,
    project_path: PathBuf,
}

/// Compute paths depending on settings and Python version
//
// This makes sure that incompatible virtualenv have different paths.
// (For instance, a "production" virtualenv must be in a different path
// than the "development" virtualenv). Ditto when the Python version changes
impl PathsResolver {
    pub fn new(project_path: PathBuf, python_version: &str, settings: &Settings) -> Self {
        PathsResolver {
            venv_outside_project: settings.venv_outside_project,
            project_path,
            python_version: python_version.into(),
            production: settings.production,
        }
    }

    pub fn paths(&self) -> Result<Paths, Error> {
        let lock_path = if self.production {
            PROD_LOCK_FILENAME
        } else {
            DEV_LOCK_FILENAME
        };
        Ok(Paths {
            project: self.project_path.clone(),
            venv: self.get_venv_path()?,
            lock: self.project_path.join(lock_path),
            setup_py: self.project_path.join("setup.py"),
        })
    }

    fn get_venv_path(&self) -> Result<PathBuf, Error> {
        if let Ok(existing_venv) = std::env::var("VIRTUAL_ENV") {
            return Ok(PathBuf::from(existing_venv));
        }
        if self.venv_outside_project {
            self.get_venv_path_outside()
        } else {
            self.get_venv_path_inside()
        }
    }

    fn get_venv_path_inside(&self) -> Result<PathBuf, Error> {
        let subdir = if self.production { "prod" } else { "dev" };
        let res = self
            .project_path
            .join(".venv")
            .join(subdir)
            .join(&self.python_version);
        Ok(res)
    }

    /// Get a suitable virtualenv path in the HOME directory.
    //
    // Note: use app_dir UserCache so that we honor XDG spec on Linux,
    // and use otherwise "expected" paths on macOS and Windows
    // (`Library/Cachches` and `AppData\Local` respectively)
    fn get_venv_path_outside(&self) -> Result<PathBuf, Error> {
        let data_dir =
            app_dirs::app_dir(AppDataType::UserCache, &APP_INFO, "venv").map_err(|e| {
                new_error(&format!(
                    "Could not create dmenv cache path: {}",
                    e.to_string()
                ))
            })?;
        let subdir = if self.production { "prod" } else { "dev" };
        let project_name = self.project_path.file_name().ok_or_else(|| {
            new_error(&format!(
                "project path: {:?} has no file name",
                self.project_path
            ))
        })?;
        let res = data_dir
            .join(subdir)
            .join(&self.python_version)
            .join(project_name);
        Ok(res)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn get_venv_path(project_path: PathBuf, settings: Settings, python_version: &str) -> PathBuf {
        let paths_resolver = PathsResolver::new(project_path, python_version, &settings);
        let paths = paths_resolver.paths().unwrap();
        paths.venv
    }

    #[test]
    fn test_resolving_paths_contains_python_version() {
        let project_path = Path::new("/tmp/foo");
        let settings = Settings::default();
        let path = get_venv_path(project_path.to_path_buf(), settings, "3.7");
        assert!(path.to_string_lossy().contains("3.7"));
    }

    #[test]
    fn test_resolving_paths_inside_by_default() {
        let project_path = Path::new("/tmp/foo");
        let settings = Settings::default();
        let path = get_venv_path(project_path.to_path_buf(), settings, "3.7");
        assert!(path.to_string_lossy().contains("/tmp/foo"));
    }

    #[test]
    fn test_resolving_paths_outside_project() {
        let project_path = Path::new("/tmp/foo");
        let settings = Settings {
            venv_outside_project: true,
            ..Default::default()
        };
        let path = get_venv_path(project_path.to_path_buf(), settings, "3.7");
        assert!(!path.to_string_lossy().contains("/tmp/foo"));
    }

    #[test]
    fn test_resolving_paths_prod_differs_from_dev() {
        let project_path = Path::new("/tmp/foo");
        let prod_settings = Settings {
            production: true,
            ..Default::default()
        };
        let prod_path = get_venv_path(project_path.to_path_buf(), prod_settings, "3.7");

        let dev_settings = Settings {
            production: false,
            ..Default::default()
        };
        let dev_path = get_venv_path(project_path.to_path_buf(), dev_settings, "3.7");

        assert_ne!(prod_path, dev_path);
    }
}
