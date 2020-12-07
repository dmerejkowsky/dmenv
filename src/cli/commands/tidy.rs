use crate::cli::commands;
use crate::cli::syntax::Command;
use crate::error::*;
use crate::operations;
use crate::{get_context, Context};

// Re-generate a clean lock:
//   - clean the virtualenv
//   - re-create it from scratch, while
//     making sure no package is updated,
//     hence the use of `pip install --constraint`
//     in `self.install_editable_with_constraint()`
//  - re-generate the lock by only keeping existing dependencies:
//    see `operations::lock::tidy()`
pub fn tidy(cmd: &Command, context: Context) -> Result<(), Error> {
    commands::clean_venv(context)?;
    // Re-create a context since we've potenntially just
    // deleted the python we used to clean the previous virtualenv
    let context = get_context(&cmd)?;
    commands::create_venv(&context)?;
    commands::install_editable_with_constraint(&context)?;
    let metadata = commands::lock_metadata(&context);
    let frozen_deps = commands::get_frozen_deps(&context)?;
    let Context { paths, .. } = context;
    operations::lock::tidy(&paths.lock, frozen_deps, &metadata)
}
