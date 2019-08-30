use colored::*;
use std::path::PathBuf;

use crate::cmd::*;
use crate::dependencies::FrozenDependency;
use crate::error::*;
use crate::lock;
use crate::lock::Updater;
use crate::lock::{git_bump, simple_bump};
use crate::project::Metadata;

#[derive(Default)]
/// Represents options passed to `dmenv lock`,
/// see `cmd::SubCommand::Lock`
pub struct LockOptions {
    pub python_version: Option<String>,
    pub sys_platform: Option<String>,
}

pub fn bump_in_lock(
    lock_path: &PathBuf,
    name: &str,
    version: &str,
    git: bool,
    metadata: &Metadata,
) -> Result<(), Error> {
    let lock_contents =
        std::fs::read_to_string(lock_path).map_err(|e| new_read_error(e, lock_path))?;
    let mut deps = lock::parse(&lock_contents)?;
    let changed = if git {
        git_bump(&mut deps, name, version)
    } else {
        simple_bump(&mut deps, name, version)
    }?;
    if !changed {
        print_warning(&format!("Dependency {} already up-to-date", name.bold()));
        return Ok(());
    }
    let new_contents = lock::dump(&deps);
    write_lock(lock_path, &new_contents, metadata)?;
    println!("{}", "ok!".green());
    Ok(())
}

// TODO: update_dependencies
pub fn lock_dependencies(
    lock_path: &PathBuf,
    frozen_deps: Vec<FrozenDependency>,
    lock_options: &LockOptions,
    metadata: &Metadata,
) -> Result<(), Error> {
    let lock_contents = if lock_path.exists() {
        std::fs::read_to_string(lock_path).map_err(|e| new_read_error(e, lock_path))?
    } else {
        String::new()
    };

    let mut updater = Updater::new();
    if let Some(python_version) = &lock_options.python_version {
        updater.python_version(&python_version);
    }
    if let Some(ref sys_platform) = lock_options.sys_platform {
        updater.sys_platform(&sys_platform);
    }
    let mut locked_deps = lock::parse(&lock_contents)?;
    updater.update(&mut locked_deps, &frozen_deps);

    let new_contents = lock::dump(&locked_deps);
    write_lock(lock_path, &new_contents, metadata)
}

pub fn write_lock(
    lock_path: &PathBuf,
    lock_contents: &str,
    metadata: &Metadata,
) -> Result<(), Error> {
    let Metadata {
        dmenv_version,
        python_version,
        python_platform,
    } = metadata;

    let top_comment = format!(
        "# Generated with dmenv {}, python {}, on {}\n",
        dmenv_version, &python_version, &python_platform
    );

    let to_write = top_comment + lock_contents;
    std::fs::write(&lock_path, to_write).map_err(|e| new_write_error(e, lock_path))
}
