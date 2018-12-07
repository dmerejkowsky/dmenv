use colored::*;

#[derive(Debug)]
pub enum Error {
    ReadError {
        path: std::path::PathBuf,
        io_error: std::io::Error,
    },
    WriteError {
        path: std::path::PathBuf,
        io_error: std::io::Error,
    },

    ProcessWaitError {
        io_error: std::io::Error,
    },
    ProcessOutError {
        io_error: std::io::Error,
    },

    MissingSetupPy {},
    MissingLock {},
    MissingVenv {
        path: std::path::PathBuf,
    },

    FileExists {
        path: std::path::PathBuf,
    },

    Other {
        message: String,
    },

    MalformedLock {
        line: usize,
        details: String,
    },

    NothingToBump {
        name: String,
    },

    MultipleBumps {
        name: String,
    },
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let message = match self {
            Error::Other { message } => message.to_string(),

            Error::ReadError { path, io_error } => {
                format!("could not read {}: {}", path.to_string_lossy(), io_error)
            }
            Error::WriteError { path, io_error } => {
                format!("could not write {}: {}", path.to_string_lossy(), io_error)
            }

            Error::ProcessWaitError { io_error } => {
                format!("could not wait for process: {}", io_error)
            }
            Error::ProcessOutError { io_error } => {
                format!("could not get process output: {}", io_error)
            }

            Error::MissingSetupPy {} => {
                "setup.py not found.\n You may want to run `dmenv init` now".to_string()
            }
            Error::MissingLock {} => {
                "requirements.lock not found.\n You may want to run `dmenv lock` now".to_string()
            }
            Error::MissingVenv { path } => {
                let mut message = format!(
                    "virtualenv in {} does not exist\n",
                    path.to_string_lossy().bold()
                );
                message.push_str("Please run `dmenv lock` or `dmenv install` to create it");
                message
            }

            Error::FileExists { path } => format!("{} already exist", path.to_string_lossy()),

            Error::MalformedLock { line, details } => {
                format!("Malformed lock at line {}\n:{}", line, details)
            }
            Error::NothingToBump { name } => format!("'{}' not found in lock", name),
            Error::MultipleBumps { name } => {
                format!("multiple matches found for '{}' in lock", name)
            }
        };
        write!(f, "{}", message)
    }
}

pub fn new<T>(message: &str) -> Result<T, Error> {
    Err(Error::Other {
        message: message.to_string(),
    })
}

pub fn write<T>(path: std::path::PathBuf, error: std::io::Error) -> Result<T, Error> {
    Err(Error::WriteError {
        io_error: error,
        path: path,
    })
}

pub fn read<T>(path: std::path::PathBuf, error: std::io::Error) -> Result<T, Error> {
    Err(Error::ReadError {
        io_error: error,
        path: path,
    })
}

pub fn process_wait<T>(error: std::io::Error) -> Result<T, Error> {
    Err(Error::ProcessWaitError { io_error: error })
}

pub fn process_out<T>(error: std::io::Error) -> Result<T, Error> {
    Err(Error::ProcessOutError { io_error: error })
}

pub fn missing_setup_py<T>() -> Result<T, Error> {
    Err(Error::MissingSetupPy {})
}

pub fn missing_lock<T>() -> Result<T, Error> {
    Err(Error::MissingLock {})
}

pub fn missing_venv<T>(path: std::path::PathBuf) -> Result<T, Error> {
    Err(Error::MissingVenv { path })
}

pub fn file_exists<T>(path: std::path::PathBuf) -> Result<T, Error> {
    Err(Error::FileExists { path })
}

pub fn malformed_lock<T>(line: usize, details: &str) -> Result<T, Error> {
    Err(Error::MalformedLock {
        line,
        details: details.to_string(),
    })
}

pub fn nothing_to_bump<T>(name: &str) -> Result<T, Error> {
    Err(Error::NothingToBump {
        name: name.to_string(),
    })
}

pub fn multiple_bumps<T>(name: &str) -> Result<T, Error> {
    Err(Error::MultipleBumps {
        name: name.to_string(),
    })
}
