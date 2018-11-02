use config::ConfigHandler;
use error::Error;

pub struct PythonsManager {
    config_handler: ConfigHandler,
}

impl PythonsManager {
    pub fn new(config_handler: ConfigHandler) -> Self {
        PythonsManager { config_handler }
    }

    pub fn add(&self, version: &str, path: &str) -> Result<(), Error> {
        self.config_handler.add_python(version, path)
    }

    pub fn remove(&self, version: &str) -> Result<(), Error> {
        self.config_handler.remove_python(version)
    }

    pub fn list(&self) -> Result<(), Error> {
        self.config_handler.list_pythons()
    }
}

#[cfg(test)]
mod tests {}
