use crate::error::*;
use crate::operations;
use crate::ui::*;
use crate::Context;

pub fn ensure_venv(context: &Context) -> Result<(), Error> {
    let Context { paths, .. } = context;
    if paths.venv.exists() {
        print_info_2(&format!(
            "Using existing virtualenv: {}",
            paths.venv.display()
        ));
    } else {
        create_venv(context)?;
    }
    Ok(())
}

/// Create a new virtualenv
//
// Notes:
// * The path comes from PathsResolver.paths()
// * Called by `ensure_venv()` *if* the path does not exist
pub fn create_venv(context: &Context) -> Result<(), Error> {
    let Context {
        paths,
        python_info,
        settings,
        ..
    } = context;
    operations::venv::create(&paths.venv, python_info, settings)
}

/// Clean virtualenv. No-op if the virtualenv does not exist
// Note: the Context is moved because if you call this function
// from inside a virtualenv, the Python binary gets invalidated
pub fn clean_venv(context: Context) -> Result<(), Error> {
    let Context { paths, .. } = context;
    operations::venv::clean(&paths.venv)
}

/// Make sure the virtualenv exists, or return an error
//
// Note: this must be called by any method that requires the
// virtualenv to exist, like `show_deps` or `run`:
// this ensures that error messages printed when the
// virtualenv does not exist are consistent.
pub fn expect_venv(context: &Context) -> Result<(), Error> {
    let Context { paths, .. } = context;
    operations::venv::expect(&paths.venv)
}
