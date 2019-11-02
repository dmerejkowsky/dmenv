use crate::cli::commands;
use crate::error::*;
use crate::lock::BumpType;
use crate::operations;
use crate::ui::*;
use crate::Context;
use crate::Metadata;

/// (Re)generate the lock file
//
// Notes:
//
// * Abort if `setup.py` is not found
// * Create the virtualenv if required
// * Always upgrade pip :
//    * If that fails, we know if the virtualenv is broken
//    * Also, we know sure that `pip` can handle all the options
//      (such as `--local`, `--exclude-editable`) we use in the other functions
// * The path of the lock file is computed by PathsResolver.
//     See PathsResolver.paths() for details
pub fn update_lock(
    context: &Context,
    update_options: operations::UpdateOptions,
) -> Result<(), Error> {
    print_info_1("Updating lock");
    let Context { paths, .. } = context;
    if !&paths.setup_py.exists() {
        return Err(Error::MissingSetupPy {});
    }
    commands::ensure_venv(&context)?;
    commands::upgrade_pip(&context)?;
    commands::install_editable(&context)?;
    let metadata = commands::metadata(&context);
    let frozen_deps = commands::get_frozen_deps(&context)?;
    let lock_path = &paths.lock;
    operations::lock::update(lock_path, frozen_deps, update_options, &metadata)
}

/// Bump a dependency in the lock file
pub fn bump_in_lock(
    context: &Context,
    name: &str,
    version: &str,
    bump_type: BumpType,
) -> Result<(), Error> {
    print_info_1(&format!("Bumping {} to {} ...", name, version));
    let metadata = commands::metadata(&context);
    let Context { paths, .. } = context;
    operations::lock::bump(&paths.lock, name, version, bump_type, &metadata)
}

pub fn metadata(context: &Context) -> Metadata {
    let Context { python_info, .. } = context;
    let dmenv_version = env!("CARGO_PKG_VERSION");
    let python_platform = &python_info.platform;
    let python_version = &python_info.version;
    Metadata {
        dmenv_version: dmenv_version.to_string(),
        python_platform: python_platform.to_string(),
        python_version: python_version.to_string(),
    }
}
