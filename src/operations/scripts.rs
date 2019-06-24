use colored::Colorize;
use glob;
use ini::Ini;
use std::path::{Path, PathBuf};

use crate::cmd;
use crate::error::*;
use crate::paths::Paths;

pub fn create(paths: &Paths) -> Result<(), Error> {
    let key = "DMENV_SCRIPTS_PATH";
    let scripts_path = std::env::var_os(key)
        .ok_or_else(|| new_error(&format!("{} environment variable not set", key)))?;
    let scripts_path = Path::new(&scripts_path);
    let egg_info_path = find_egg_info(&paths.project)?;
    let console_scripts = read_entry_points(&egg_info_path)?;
    cmd::print_info_1(&format!(
        "found {} console script(s)",
        console_scripts.len()
    ));
    for console_script in console_scripts {
        process(&paths.venv, &scripts_path.to_path_buf(), console_script)?;
    }
    Ok(())
}

fn process(
    venv_path: &PathBuf,
    scripts_path: &PathBuf,
    console_script: ConsoleScript,
) -> Result<(), Error> {
    let src_path = venv_path.join("bin").join(&console_script.name);
    let dest_path = scripts_path.join(&console_script.name);
    cmd::print_info_2(&format!(
        "Creating script {} calling {}",
        console_script.name.bold(),
        console_script.value.bold(),
    ));
    if !src_path.exists() {
        return Err(new_error(&format!(
            "{:?} does not exist. You may want to call `dmenv develop` now",
            src_path
        )));
    }
    #[cfg(windows)]
    {
        safe_copy(&src_path, &dest_path)
    }
    #[cfg(unix)]
    {
        safe_create_symlink(&src_path, &dest_path)
    }
}

#[cfg(windows)]
fn safe_copy(src_path: &PathBuf, dest_path: &PathBuf) -> Result<(), Error> {
    if dest_path.exists() {
        return Err(new_error(&format!("{:?} already exists", &src_path)));
    }
    std::fs::copy(src_path, dest_path).map_err(|e| 
    new_error(&format!(
    "Could not copy from {:?} to {:?}: {}",
        src_path, dest_path, e
    )))?;
    Ok(())
}

#[cfg(unix)]
fn safe_create_symlink(src_path: &PathBuf, dest_path: &PathBuf) -> Result<(), Error> {
    delete_if_link(&dest_path)?;
    println!(
        "{} -> {}",
        dest_path.to_string_lossy(),
        src_path.to_string_lossy()
    );
    std::os::unix::fs::symlink(&src_path, &dest_path).map_err(|e| {
        new_error(&format!(
            "Could not create link from {:?} to {:?}: {}",
            dest_path, src_path, e
        ))
    })
}

fn delete_if_link(path: &PathBuf) -> Result<(), Error> {
    // This will make an error if the path does not exist,
    // but also if we don't have permission to read the path)
    // In both cases, we want to return early - in the latter
    // code will probably fail when creating the symlink
    // later on.
    let meta = std::fs::symlink_metadata(path);
    if meta.is_err() {
        return Ok(());
    };
    let meta = meta.unwrap();
    if !meta.file_type().is_symlink() {
        return Err(new_error(&format!(
            "{:?} exists and is *not* a symlink",
            path
        )));
    };
    std::fs::remove_file(path).map_err(|e| {
        new_error(&format!(
            "Could not remove existing symlink {:?}: {}",
            path, e
        ))
    })?;
    Ok(())
}

fn find_egg_info(project_path: &PathBuf) -> Result<PathBuf, Error> {
    let pattern = format!("{}/*.egg-info", project_path.to_string_lossy());
    let glob = glob::glob(&pattern).expect("could not parse glob pattern");
    let mut matches = vec![];
    for entry in glob {
        if let Ok(path) = entry {
            matches.push(path)
        }
    }
    let num_matches = matches.len();
    if num_matches != 1 {
        return Err(new_error(&format!(
            "Expecting exactly one .egg-info entry, got {}",
            num_matches
        )));
    }
    Ok(matches[0].clone())
}

struct ConsoleScript {
    name: String,
    value: String,
}

type ConsoleScripts = Vec<ConsoleScript>;

fn read_entry_points(egg_info_path: &PathBuf) -> Result<ConsoleScripts, Error> {
    let entry_points_txt_path = egg_info_path.join("entry_points.txt");
    let config = Ini::load_from_file(&entry_points_txt_path).map_err(|e| {
        new_error(&format!(
            "Could not read {:?}: {}",
            &entry_points_txt_path, e
        ))
    })?;
    let mut res = vec![];
    let section = config.section(Some("console_scripts"));
    if let Some(section) = section {
        for (key, value) in section.iter() {
            let cs = ConsoleScript {
                name: key.to_string(),
                value: value.to_string(),
            };
            res.push(cs);
        }
    }
    Ok(res)
}
