use colored::*;
use std::path::Path;

use crate::dependencies::FrozenDependency;
use crate::error::*;
use crate::lock;
use crate::lock::Updater;
use crate::lock::{git_bump, simple_bump};
use crate::ui::*;
use crate::BumpType;
use crate::Metadata;
use crate::UpdateOptions;

pub fn bump(
    lock_path: &Path,
    name: &str,
    version: &str,
    bump_type: BumpType,
    metadata: &Metadata,
) -> Result<(), Error> {
    let lock_contents =
        std::fs::read_to_string(lock_path).map_err(|e| new_read_error(e, lock_path))?;
    let mut deps = lock::parse(&lock_contents)?;
    let changed = match bump_type {
        BumpType::Git => git_bump(&mut deps, name, version),
        BumpType::Simple => simple_bump(&mut deps, name, version),
    }?;
    if !changed {
        print_warning(&format!("Dependency {} already up-to-date", name.bold()));
        return Ok(());
    }
    let new_contents = lock::dump(deps);
    write_lock(lock_path, &new_contents, metadata)?;
    println!("{}", "ok!".green());
    Ok(())
}

pub fn update(
    lock_path: &Path,
    frozen_deps: Vec<FrozenDependency>,
    update_options: UpdateOptions,
    metadata: &Metadata,
) -> Result<(), Error> {
    print_info_2(&format!("Generating {}", lock_path.display()));
    let lock_contents = if lock_path.exists() {
        std::fs::read_to_string(lock_path).map_err(|e| new_read_error(e, lock_path))?
    } else {
        String::new()
    };

    let mut updater = Updater::new();
    updater.set_options(update_options);
    let mut locked_deps = lock::parse(&lock_contents)?;
    updater.update(&mut locked_deps, &frozen_deps);

    let new_contents = lock::dump(locked_deps);
    write_lock(lock_path, &new_contents, metadata)
}

pub fn tidy(
    lock_path: &Path,
    frozen_deps: Vec<FrozenDependency>,
    metadata: &Metadata,
) -> Result<(), Error> {
    print_info_2(&format!("Tidying {}", lock_path.display()));
    let frozen_names: Vec<_> = frozen_deps.into_iter().map(|x| x.name).collect();
    let lock_contents =
        std::fs::read_to_string(lock_path).map_err(|e| new_read_error(e, lock_path))?;
    let locked_deps = lock::parse(&lock_contents)?;

    // Remove from the lock file anything that is _not_ in the clean virtualenv
    let (deps_to_keep, deps_to_remove): (Vec<_>, Vec<_>) = locked_deps
        .into_iter()
        .partition(|x| frozen_names.contains(&x.name()));
    for dep in deps_to_remove {
        println!("- {}", dep.name());
    }
    let new_contents = &lock::dump(deps_to_keep);
    write_lock(lock_path, new_contents, metadata)
}

pub fn write_lock(lock_path: &Path, lock_contents: &str, metadata: &Metadata) -> Result<(), Error> {
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
