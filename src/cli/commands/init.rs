use std::path::PathBuf;

use crate::error::*;
use crate::operations;

pub fn init(
    project_path: Option<String>,
    name: &str,
    version: &str,
    author: &Option<String>,
    setup_cfg: bool,
) -> Result<(), Error> {
    let init_path = if let Some(p) = project_path {
        PathBuf::from(p)
    } else {
        std::env::current_dir().map_err(|e| Error::NoWorkingDirectory { io_error: e })?
    };

    let mut init_options = operations::InitOptions::new(name.to_string(), version.to_string());
    if !setup_cfg {
        init_options.no_setup_cfg();
    };
    if let Some(author) = author {
        init_options.author(&author);
    }
    operations::init(&init_path, &init_options)
}
