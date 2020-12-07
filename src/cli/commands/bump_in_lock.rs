use crate::cli::commands;
use crate::error::*;
use crate::operations;
use crate::ui::*;
use crate::BumpType;
use crate::Context;

/// Bump a dependency in the lock file
pub fn bump_in_lock(
    context: &Context,
    name: &str,
    version: &str,
    bump_type: BumpType,
) -> Result<(), Error> {
    print_info_1(&format!("Bumping {} to {} ...", name, version));
    let metadata = commands::lock_metadata(&context);
    let Context { paths, .. } = context;
    operations::lock::bump(&paths.lock, name, version, bump_type, &metadata)
}
