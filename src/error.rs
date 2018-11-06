#[derive(Debug)]
pub struct Error {
    description: String,
}

impl Error {
    pub fn new(description: &str) -> Error {
        Error {
            description: String::from(description),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Error {
        Error::new(&format!("I/O error: {}", error))
    }
}

impl From<toml::ser::Error> for Error {
    fn from(error: toml::ser::Error) -> Error {
        Error::new(&format!("Could not serialize config: {}", error))
    }
}

impl From<toml::de::Error> for Error {
    fn from(error: toml::de::Error) -> Error {
        Error::new(&format!("Could not parse config: {}", error))
    }
}

impl From<which::Error> for Error {
    fn from(error: which::Error) -> Error {
        Error::new(&error.to_string())
    }
}

impl From<std::env::JoinPathsError> for Error {
    fn from(error: std::env::JoinPathsError) -> Error {
        let mut message =
            "The computed new PATH contains invalid characters. Please open a bug report"
                .to_string();
        message.push_str(&format!("\nOriginal error: {}", error));
        Error::new(&message)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", &self.description)
    }
}
