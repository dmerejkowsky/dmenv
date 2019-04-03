use crate::cmd::*;
use crate::error::Error;
use std::path::PathBuf;

pub struct InitOptions {
    name: String,
    version: String,
    author: Option<String>,
    setup_cfg: bool,
}

impl InitOptions {
    pub fn new(name: &str, version: &str, author: &Option<String>) -> Self {
        InitOptions {
            name: name.to_string(),
            version: version.to_string(),
            author: author.as_ref().map(|x| x.to_string()),
            setup_cfg: true,
        }
    }

    pub fn no_setup_cfg(&mut self) {
        self.setup_cfg = false
    }
}

fn ensure_path_does_not_exist(path: &PathBuf) -> Result<(), Error> {
    if path.exists() {
        return Err(Error::FileExists {
            path: path.to_path_buf(),
        });
    }
    Ok(())
}

fn write_to_path(path: &PathBuf, contents: &str) -> Result<(), Error> {
    std::fs::write(path, contents).map_err(|e| Error::WriteError {
        path: path.to_path_buf(),
        io_error: e,
    })
}

pub fn init(project_path: &PathBuf, options: &InitOptions) -> Result<(), Error> {
    let setup_cfg_path = project_path.join("setup.cfg");
    let setup_py_path = project_path.join("setup.py");

    // A setup.py is written in both cases, so check we're not
    // overwriting it first
    ensure_path_does_not_exist(&setup_py_path)?;

    if options.setup_cfg {
        ensure_path_does_not_exist(&setup_cfg_path)?;
        write_from_template(include_str!("setup.in.cfg"), &setup_cfg_path, options)?;
    } else {
        write_from_template(include_str!("setup.in.py"), &setup_py_path, options)?;
    }

    if options.setup_cfg {
        // We still need an almost-empty setup.py even when all the configuration
        // is in setup.cfg :/
        write_to_path(&setup_py_path, "from setuptools import setup\nsetup()\n")?;
        print_info_1("Project initialized with setup.py and setup.cfg files");
    } else {
        print_info_1("Project initialized with a setup.py file");
    }
    Ok(())
}

fn write_from_template(
    template: &str,
    dest_path: &PathBuf,
    options: &InitOptions,
) -> Result<(), Error> {
    // Warning: make sure the template files in `src/operations/` contain all those
    // placeholders
    let with_name = template.replace("<NAME>", &options.name);
    let with_version = with_name.replace("<VERSION>", &options.version);
    let to_write = if let Some(ref author) = options.author {
        with_version.replace("<AUTHOR>", author)
    } else {
        with_version
    };
    write_to_path(dest_path, &to_write)
}
