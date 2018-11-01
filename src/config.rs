extern crate colored;
extern crate serde;
extern crate serde_derive;
use std::collections::BTreeMap as Map;

use error::Error;

#[derive(Deserialize)]
pub struct Config {
    env: Map<String, Env>,
}

#[derive(Deserialize)]
struct Env {
    python: String,
}

// TODO: config struct with a new() that parses!
pub fn parse_config() -> Result<Config, Error> {
    let config_dir = appdirs::user_config_dir(None, None, false);
    // The type is Result<PathBuf, ()> I blame upstream
    if config_dir.is_err() {
        return Err(Error::new(
            "appdirs::user_data_dir() failed. That's all we know",
        ));
    }
    let config_dir = config_dir.unwrap();
    let cfg_path = config_dir.join("dmenv.toml");
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

pub fn get_python_for_env(config: Config, env_name: &str) -> Result<String, Error> {
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
        let config = toml::from_str(&config).unwrap();
        let actual = super::get_python_for_env(config, "3.8").unwrap();
        assert_eq!(actual, "/path/to/python3.8");

        let actual = super::get_python_for_env(config, "nosuch");
        assert!(actual.is_err());
    }
}
