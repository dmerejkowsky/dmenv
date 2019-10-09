use std::path::{Path, PathBuf};

/// Every variant matches a type of error we
/// want the end-use to see.
// Note: errors from external crates should be wrapped
/// here so that we have full control over the error
/// messages printed to the user.
#[derive(Debug)]
pub enum Error {
    ReadError {
        path: PathBuf,
        io_error: std::io::Error,
    },
    WriteError {
        path: PathBuf,
        io_error: std::io::Error,
    },

    NoWorkingDirectory {
        io_error: std::io::Error,
    },

    NulByteFound {
        arg: String,
    },
    ProcessStartError {
        message: String,
    },
    ProcessWaitError {
        io_error: std::io::Error,
    },
    ProcessOutError {
        io_error: std::io::Error,
    },

    InfoPyError {
        message: String,
    },

    PipUpgradeFailed {},
    BrokenPipFreezeLine {
        line: String,
    },

    MissingSetupPy {},
    MissingLock {
        expected_path: PathBuf,
    },
    MissingVenv {
        path: PathBuf,
    },

    FileExists {
        path: PathBuf,
    },

    Other {
        message: String,
    },

    MalformedLock {
        details: String,
    },

    NothingToBump {
        name: String,
    },

    MultipleBumps {
        name: String,
    },
    IncorrectLockedType {
        name: String,
        expected_type: String,
    },
}

pub fn new_error(message: &str) -> Error {
    Error::Other {
        message: message.to_string(),
    }
}

pub fn new_read_error(error: std::io::Error, path: &Path) -> Error {
    Error::ReadError {
        path: path.to_path_buf(),
        io_error: error,
    }
}

pub fn new_write_error(error: std::io::Error, path: &Path) -> Error {
    Error::WriteError {
        path: path.to_path_buf(),
        io_error: error,
    }
}

/// Implement Display for our Error type
// Note: this is a not-so-bad way to make sure every error message is consistent
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let message = match self {
            Error::Other { message } => message.to_string(),

            Error::NulByteFound { arg } => format!("nul byte found in arg: {:?}", arg),

            Error::ReadError { path, io_error } => {
                format!("could not read {}: {}", path.display(), io_error)
            }
            Error::WriteError { path, io_error } => {
                format!("could not write {}: {}", path.display(), io_error)
            }

            Error::NoWorkingDirectory { io_error } => {
                format!("could not get current working directory: {}", io_error)
            }


            Error::ProcessStartError { message } => format!("could not start process: {}", message),
            Error::ProcessWaitError { io_error } => {
                format!("could not wait for process: {}", io_error)
            }
            Error::ProcessOutError { io_error } => {
                format!("could not get process output: {}", io_error)
            }

            Error::InfoPyError { message } => {
                format!("could not determine Python version and platform while running the `info.py` script: {}",
                      message)
            },

            Error::MissingSetupPy {} => {
                "setup.py not found.\nYou may want to run `dmenv init` now".to_string()
            }
            Error::MissingLock { expected_path } => format!(
                "{} not found.\nYou may want to run `dmenv lock` now",
                expected_path.display()
            ),
            Error::MissingVenv { path } => {
                let mut message = format!("virtualenv in '{}' does not exist\n", path.display());
                message.push_str("Please run `dmenv lock` or `dmenv install` to create it");
                message
            }

            Error::BrokenPipFreezeLine { line } => {
                format!("could not parse `pip freeze` output at line: '{}'", line)
            }
            Error::PipUpgradeFailed {} => {
                "could not upgrade pip. Try using `dmenv clean`".to_string()
            }

            Error::FileExists { path } => format!("{} already exists", path.display()),

            Error::MalformedLock { details } => format!("Malformed lock: {}", details),

            Error::NothingToBump { name } => format!("'{}' not found in lock", name),
            Error::MultipleBumps { name } => {
                format!("multiple matches found for '{}' in lock", name)
            }
            Error::IncorrectLockedType {
                name,
                expected_type,
            } => format!("{} is not a {} dependency", name, expected_type),
        };
        write!(f, "{}", message)
    }
}
