use std::path::PathBuf;

use crate::cmd::*;
use crate::error::*;
use crate::python_info::PythonInfo;
use crate::run::run;
use crate::settings::Settings;

pub fn clean(venv_path: PathBuf) -> Result<(), Error> {
    print_info_1(&format!("Cleaning {}", venv_path.display()));
    if !venv_path.exists() {
        return Ok(());
    }
    std::fs::remove_dir_all(&venv_path)
        .map_err(|e| new_error(&format!("could not remove {}: {}", venv_path.display(), e)))?;
    Ok(())
}

pub fn create(
    venv_path: &PathBuf,
    python_info: &PythonInfo,
    settings: &Settings,
) -> Result<(), Error> {
    let parent_venv_path = venv_path
        .parent()
        .ok_or_else(|| new_error("venv_path has no parent"))?;
    print_info_2(&format!("Creating virtualenv in: {}", venv_path.display()));
    std::fs::create_dir_all(&parent_venv_path).map_err(|e| {
        new_error(&format!(
            "Could not create {}: {}",
            parent_venv_path.display(),
            e
        ))
    })?;

    // Python -m venv should work in most cases (venv is in the stdlib since Python 3.3)
    let venv_path_str: String = venv_path.to_string_lossy().into();
    let mut args = vec!["-m"];
    if settings.venv_from_stdlib {
        args.push("venv")
    } else {
        // In case we can't or won't use venv from the stdlib, use `virtualenv` instead.
        // Assume the virtualenv package is present on the system.
        args.push("virtualenv")
    };
    args.push(&venv_path_str);
    if settings.system_site_packages {
        args.push("--system-site-packages");
    }
    let python_binary = &python_info.binary;
    run(&parent_venv_path.to_path_buf(), &python_binary, args)
}

pub fn expect(venv_path: &PathBuf) -> Result<(), Error> {
    if !venv_path.exists() {
        return Err(Error::MissingVenv {
            path: venv_path.clone(),
        });
    }
    Ok(())
}
