use colored::Colorize;
use ini::Ini;
use std::path::{Path, PathBuf};

use crate::cmd;
use crate::error::*;
use crate::paths::{Paths, SCRIPTS_SUBDIR};
use crate::ProcessScriptsMode::{self, Override, Safe};

pub fn process(paths: &Paths, mode: ProcessScriptsMode) -> Result<(), Error> {
    let key = "DMENV_SCRIPTS_PATH";
    let scripts_path = std::env::var_os(key)
        .ok_or_else(|| new_error(format!("{} environment variable not set", key)))?;
    let scripts_path = Path::new(&scripts_path);
    let egg_info_path = find_egg_info(&paths.project)?;
    let console_scripts = read_entry_points(&egg_info_path)?;
    cmd::print_info_1(&format!(
        "found {} console script(s)",
        console_scripts.len()
    ));
    for console_script in console_scripts {
        process_script(
            &paths.venv,
            &scripts_path.to_path_buf(),
            &console_script,
            mode,
        )?;
    }
    Ok(())
}

fn process_script(
    venv_path: &Path,
    scripts_path: &Path,
    entry_point_name: &str,
    mode: ProcessScriptsMode,
) -> Result<(), Error> {
    #[cfg(unix)]
    let names = [entry_point_name];

    #[cfg(windows)]
    let names = [
        format!("{}.exe", entry_point_name),
        format!("{}-script.py", entry_point_name),
    ];

    for name in names.iter() {
        process_script_with_name(venv_path, scripts_path, &name, mode)?;
    }
    Ok(())
}

fn process_script_with_name(
    venv_path: &Path,
    scripts_path: &Path,
    name: &str,
    mode: ProcessScriptsMode,
) -> Result<(), Error> {
    let src_path = venv_path.join(SCRIPTS_SUBDIR).join(name);
    let dest_path = scripts_path.join(name);
    cmd::print_info_2(&format!("Creating script: {}", name.bold()));
    if !src_path.exists() {
        return Err(new_error(format!(
            "{} does not exist. You may want to call `dmenv develop` now",
            src_path.display()
        )));
    }
    #[cfg(windows)]
    {
        match mode {
            Safe => safe_copy(&src_path, &dest_path),
            Override => copy(&src_path, &dest_path),
        }
    }
    #[cfg(unix)]
    {
        symlink(&src_path, &dest_path, mode)
    }
}

#[cfg(windows)]
fn safe_copy(src_path: &Path, dest_path: &Path) -> Result<(), Error> {
    if dest_path.exists() {
        return Err(new_error(format!("{} already exists", src_path.display())));
    }
    copy(src_path, dest_path)
}

#[cfg(windows)]
fn copy(src_path: &Path, dest_path: &Path) -> Result<(), Error> {
    std::fs::copy(src_path, dest_path).map_err(|e| {
        new_error(format!(
            "Could not copy from {} to {}: {}",
            src_path.display(),
            dest_path.display(),
            e
        ))
    })?;
    Ok(())
}

#[cfg(unix)]
fn symlink(src_path: &Path, dest_path: &Path, mode: ProcessScriptsMode) -> Result<(), Error> {
    match mode {
        // Note: we assume it is "safe" to change the target of an existing
        // symlink
        Safe => delete_if_link(dest_path),
        // Note: we need to delete dest_path for unix::fs::symlink to work
        // later on
        Override => delete_if_exists(dest_path),
    }?;
    println!("{} -> {}", dest_path.display(), src_path.display());
    std::os::unix::fs::symlink(src_path, dest_path).map_err(|e| {
        new_error(format!(
            "Could not create link from {} to {}: {}",
            dest_path.display(),
            src_path.display(),
            e
        ))
    })
}

#[cfg(unix)]
fn delete_if_link(path: &Path) -> Result<(), Error> {
    // This will make an error if the path does not exist,
    // but also if we don't have permission to read the path.
    // In both cases, we want to return early - in the later,
    // the code will probably fail when creating the symlink
    // later on.
    let meta = std::fs::symlink_metadata(path);
    if meta.is_err() {
        return Ok(());
    };
    let meta = meta.unwrap();
    if !meta.file_type().is_symlink() {
        return Err(new_error(format!(
            "{} exists and is *not* a symlink",
            path.display()
        )));
    };
    std::fs::remove_file(path).map_err(|e| {
        new_error(format!(
            "Could not remove existing symlink {}: {}",
            path.display(),
            e
        ))
    })?;
    Ok(())
}

#[cfg(unix)]
fn delete_if_exists(path: &Path) -> Result<(), Error> {
    if path.exists() {
        std::fs::remove_file(path).map_err(|e| {
            new_error(format!(
                "Could not remove existing file {}: {}",
                path.display(),
                e
            ))
        })?;
    }
    Ok(())
}

fn list_egg_info_dirs(project_path: &Path) -> Result<Vec<PathBuf>, std::io::Error> {
    let mut res = vec![];
    for entry in std::fs::read_dir(&project_path)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            // See https://github.com/rust-lang/rfcs/issues/900
            // for why we don't use directly path.file_name().ends_with:
            // OsStr does not have ends_with
            let file_name = path.file_name().unwrap().to_string_lossy();
            if file_name.ends_with(".egg-info") {
                res.push(path);
            }
        }
    }
    Ok(res)
}

fn find_egg_info(project_path: &Path) -> Result<PathBuf, Error> {
    let matches = list_egg_info_dirs(project_path)
        .map_err(|e| new_error(format!("While listing project path: {}", e)))?;
    let num_matches = matches.len();
    if num_matches != 1 {
        return Err(new_error(format!(
            "Expecting exactly one .egg-info entry, got {}",
            num_matches
        )));
    }
    Ok(matches[0].clone())
}

fn read_entry_points(egg_info_path: &Path) -> Result<Vec<String>, Error> {
    let entry_points_txt_path = egg_info_path.join("entry_points.txt");
    let config = Ini::load_from_file(&entry_points_txt_path).map_err(|e| {
        new_error(format!(
            "Could not read {}: {}",
            &entry_points_txt_path.display(),
            e
        ))
    })?;
    let mut res = vec![];
    let section = config.section(Some("console_scripts"));
    if let Some(section) = section {
        for key in section.keys() {
            res.push(key.to_string());
        }
    }
    Ok(res)
}
