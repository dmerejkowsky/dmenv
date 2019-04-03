use crate::cmd::*;
use crate::error::Error;
use std::path::PathBuf;

pub fn init(
    project_path: &PathBuf,
    name: &str,
    version: &str,
    author: &Option<String>,
) -> Result<(), Error> {
    let path = project_path.join("setup.py");
    if path.exists() {
        return Err(Error::FileExists {
            path: path.to_path_buf(),
        });
    }
    // Warning: make sure the source file in `src/operations/setup.in.py` contains all those
    // placeholders
    let template = include_str!("setup.in.py");
    let with_name = template.replace("<NAME>", name);
    let with_version = with_name.replace("<VERSION>", version);
    let to_write = if let Some(author) = author {
        with_version.replace("<AUTHOR>", author)
    } else {
        with_version
    };
    std::fs::write(&path, to_write).map_err(|e| Error::WriteError {
        path: path.to_path_buf(),
        io_error: e,
    })?;
    print_info_1("Generated a new setup.py");
    Ok(())
}
