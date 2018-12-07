use error;
use error::Error;

pub struct PythonInfo {
    pub binary: std::path::PathBuf,
    pub version: String,
    pub platform: String,
}

impl PythonInfo {
    pub fn new(python: &Option<String>) -> Result<Self, Error> {
        let binary = get_python_binary(&python)?;
        let info_script = include_str!("info.py");

        let command = std::process::Command::new(&binary)
            .args(&["-c", info_script])
            .output();
        if let Err(e) = command {
            return error::process_out(e);
        }

        let command = command.unwrap();
        if !command.status.success() {
            return error::new(&format!(
                "Failed to run info script: {}",
                String::from_utf8_lossy(&command.stderr)
            ));
        }
        let info_out = String::from_utf8_lossy(&command.stdout);
        let lines: Vec<_> = info_out.split('\n').collect();
        if lines.len() != 3 {
            return error::new(&format!(
                "Expected two lines in info_out, got: {}",
                lines.len()
            ));
        }
        let version = lines[0].trim().to_string();
        let platform = lines[1].trim().to_string();
        Ok(PythonInfo {
            binary,
            version,
            platform,
        })
    }
}

fn get_python_binary(requested_python: &Option<String>) -> Result<std::path::PathBuf, Error> {
    if let Some(python) = requested_python {
        return Ok(std::path::PathBuf::from(python));
    }

    let python3 = which::which("python3");
    if python3.is_ok() {
        return Ok(python3.unwrap());
    }

    // Python3 may be called 'python', for instance on Windows
    let res = which::which("python");
    if res.is_err() {
        return error::new("Neither `python3` nor `python` fonud in PATH");
    }
    Ok(res.unwrap())
}
