extern crate colored;
extern crate serde;
extern crate serde_derive;
use std::collections::BTreeMap as Map;

use error::Error;

#[derive(Deserialize)]
pub struct Config {
    pythons: Map<String, String>,
}

pub struct ConfigHandler {
    config: Config,
}

impl ConfigHandler {
    pub fn new(cfg_path: Option<String>) -> Result<Self, Error> {
        let cfg_path = Self::get_config_path(cfg_path)?;
        let config = Self::parse_config(cfg_path)?;
        Ok(ConfigHandler { config })
    }

    pub fn get_python(&self, version: &str) -> Result<String, Error> {
        let matching_python = &self.config.pythons.get(version);
        if matching_python.is_none() {
            return Err(Error::new(&format!(
                "No python found matching version: {}",
                version
            )));
        }

        let matching_python = matching_python.unwrap();
        Ok(matching_python.clone())
    }

    fn get_config_path(cfg_path: Option<String>) -> Result<std::path::PathBuf, Error> {
        if cfg_path.is_some() {
            return Ok(cfg_path.unwrap().into());
        }

        let config_dir = appdirs::user_config_dir(None, None, false);
        // The type is Result<PathBuf, ()> I blame upstream
        if config_dir.is_err() {
            return Err(Error::new(
                "appdirs::user_data_dir() failed. That's all we know",
            ));
        }
        let config_dir = config_dir.unwrap();
        Ok(config_dir.join("dmenv.toml"))
    }

    fn parse_config(cfg_path: std::path::PathBuf) -> Result<Config, Error> {
        let config_str = std::fs::read_to_string(&cfg_path);
        if let Err(error) = config_str {
            return Err(Error::new(&format!(
                "Could not read from {}: {}",
                cfg_path.to_string_lossy(),
                error
            )));
        }
        let config_str = config_str.unwrap();
        let config = toml::from_str(&config_str)?;
        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_handler() {
        let tmp_dir = tempdir::TempDir::new("test-dmenv").expect("");
        let config = r#"
        [pythons]
        default = "/usr/bin/python3"
        "3.8" = "/path/to/python3.8"
        "#;
        let cfg_path = tmp_dir.path().join("dmenv.cfg");
        std::fs::write(&cfg_path, config).expect("");

        let cfg_path = cfg_path.to_string_lossy();
        let config_handler = ConfigHandler::new(Some(cfg_path.to_string())).unwrap();

        let actual = config_handler.get_python("3.8").unwrap();
        assert_eq!(actual, "/path/to/python3.8");

        let actual = config_handler.get_python("nosuch");
        assert!(actual.is_err())
    }
}
