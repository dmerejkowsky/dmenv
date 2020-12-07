use crate::cli::commands;
use crate::error::*;
use crate::operations;
use crate::ui::*;
use crate::{Context, UpdateLockOptions};

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
pub fn update_lock(context: &Context, update_options: UpdateLockOptions) -> Result<(), Error> {
    print_info_1("Updating lock");
    let Context { paths, .. } = context;
    if !&paths.setup_py.exists() {
        return Err(Error::MissingSetupPy {});
    }
    commands::ensure_venv(&context)?;
    commands::upgrade_pip(&context)?;
    commands::install_editable(&context)?;
    let metadata = commands::lock_metadata(&context);
    let frozen_deps = commands::get_frozen_deps(&context)?;
    let lock_path = &paths.lock;
    operations::lock::update(lock_path, frozen_deps, update_options, &metadata)
}
