extern crate colored;
extern crate serde;
extern crate serde_derive;
use colored::*;
use std::collections::BTreeMap as Map;

use error::Error;

#[derive(Deserialize, Serialize, Default)]
pub struct Config {
    pythons: Map<String, String>,
}

pub struct ConfigHandler {
    cfg_path: std::path::PathBuf,
}

impl ConfigHandler {
    pub fn new(cfg_path: Option<String>) -> Result<Self, Error> {
        let cfg_path = Self::get_config_path(cfg_path)?;
        return Ok(ConfigHandler { cfg_path });
    }

    pub fn get_python(&self, version: &str) -> Result<String, Error> {
        self.check_cfg_path()?;
        let config = Self::parse_config(&self.cfg_path)?;
        let matching_python = &config.pythons.get(version);
        if matching_python.is_none() {
            return Err(Error::new(&format!(
                "No python found matching version: {}",
                version
            )));
        }

        let matching_python = matching_python.unwrap();
        Ok(matching_python.clone())
    }

    pub fn add_python(&self, version: &str, path: &str) -> Result<(), Error> {
        let mut config = if self.cfg_path.exists() {
            Self::parse_config(&self.cfg_path)?
        } else {
            Config::default()
        };
        config.pythons.insert(version.to_string(), path.to_string());
        self.write_config(config)
    }

    pub fn remove_python(&self, version: &str) -> Result<(), Error> {
        self.check_cfg_path()?;
        let mut config = Self::parse_config(&self.cfg_path)?;
        config.pythons.remove(version);
        self.write_config(config)
    }

    pub fn list_pythons(&self) -> Result<(), Error> {
        self.check_cfg_path()?;
        let config = Self::parse_config(&self.cfg_path)?;
        for (version, path) in &config.pythons {
            println!("{}: {}", version.bold(), path);
        }
        Ok(())
    }

    fn check_cfg_path(&self) -> Result<(), Error> {
        if !&self.cfg_path.exists() {
            let message = format!(
                "{}\n{}", "No pythons configured yet",
                "Please run `dmenv pythons add default <path/to/python3/interpreter>`"
                );
            return Err(Error::new(&message));
        }
        Ok(())
    }

    fn get_config_path(cfg_path: Option<String>) -> Result<std::path::PathBuf, Error> {
        if cfg_path.is_some() {
            return Ok(cfg_path.unwrap().into());
        }

        let config_dir = appdirs::user_config_dir(Some("dmenv"), Some("dmenv"), false);
        // The type is Result<PathBuf, ()> I blame upstream
        if config_dir.is_err() {
            return Err(Error::new(
                "appdirs::user_data_dir() failed. That's all we know",
            ));
        }
        let config_dir = config_dir.unwrap();
        let res = config_dir.join("dmenv.toml");
        println!("Using config from {}", res.to_string_lossy());
        Ok(res)
    }

    fn write_config(&self, config: Config) -> Result<(), Error> {
        let contents = toml::to_string(&config)?;
        std::fs::write(&self.cfg_path, &contents)?;
        Ok(())
    }

    fn parse_config(cfg_path: &std::path::PathBuf) -> Result<Config, Error> {
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

    fn new_config_handler(tmp_dir: &tempdir::TempDir, contents: Option<&str>) -> ConfigHandler {
        let cfg_path = tmp_dir.path().join("dmenv.cfg");
        if let Some(contents) = contents {
            std::fs::write(&cfg_path, contents).expect("");
        }
        let cfg_path = cfg_path.to_string_lossy();
        ConfigHandler::new(Some(cfg_path.to_string())).unwrap()
    }

    #[test]
    fn test_read_pythons() {
        let tmp_dir = tempdir::TempDir::new("test-dmenv").expect("");
        let config = r#"
        [pythons]
        default = "/usr/bin/python3"
        "3.8" = "/path/to/python3.8"
        "#;
        let config_handler = new_config_handler(&tmp_dir, Some(config));

        let actual = config_handler.get_python("3.8").unwrap();
        assert_eq!(actual, "/path/to/python3.8");

        let actual = config_handler.get_python("nosuch");
        assert!(actual.is_err())
    }

    #[test]
    fn test_remove_python() {
        let tmp_dir = tempdir::TempDir::new("test-dmenv").expect("");
        let config = r#"
        [pythons]
        default = "/usr/bin/python3"
        "2.7" = "/usr/bin/python2"
        "#;
        let config_handler = new_config_handler(&tmp_dir, Some(config));

        config_handler.remove_python("2.7").expect("");

        let err = config_handler.get_python("2.7").unwrap_err();
        assert_eq!(err.to_string(), "No python found matching version: 2.7");
    }

    #[test]
    fn test_write_pythons() {
        let tmp_dir = tempdir::TempDir::new("test-dmenv").expect("");
        let config_handler = new_config_handler(&tmp_dir, None);
        config_handler
            .add_python("default", "/usr/bin/python3")
            .expect("");
        assert_eq!(
            config_handler.get_python("default").expect(""),
            "/usr/bin/python3"
        );
    }

}
