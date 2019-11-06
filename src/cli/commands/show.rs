use crate::cli::commands;
use crate::error::*;
use crate::Context;

/// Show the dependencies inside the virtualenv.
// Note: Run `pip list` so we get what's *actually* installed, not just
// the contents of the lock file
pub fn show_deps(context: &Context) -> Result<(), Error> {
    let Context { venv_runner, .. } = context;
    venv_runner.run(&["python", "-m", "pip", "list"])
}

pub fn show_outdated(context: &Context) -> Result<(), Error> {
    let Context { venv_runner, .. } = context;
    #[rustfmt::skip]
    let cmd = &[
        "python", "-m", "pip",
        "list", "--outdated",
        "--format", "columns",
    ];
    venv_runner.run(cmd)
}

/// Show the resolved virtualenv path.
//
// See `PathsResolver.paths()` for details
pub fn show_venv_path(context: &Context) -> Result<(), Error> {
    let Context { paths, .. } = context;
    println!("{}", paths.venv.display());
    Ok(())
}

/// Same has `show_venv_path`, but add the correct subfolder
/// (`bin` on Linux and macOS, `Scripts` on Windows).
pub fn show_venv_bin_path(context: &Context) -> Result<(), Error> {
    let Context { venv_runner, .. } = context;
    commands::expect_venv(&context)?;
    let bin_path = venv_runner.binaries_path();
    println!("{}", bin_path.display());
    Ok(())
}
