use crate::cli::commands;
use crate::error::*;
use crate::Context;

/// Run a program from the virtualenv, making sure it dies
/// when we get killed and that the exit code is forwarded
pub fn run_and_die<T: AsRef<str>>(context: &Context, cmd: &[T]) -> Result<(), Error> {
    let Context { venv_runner, .. } = context;
    commands::expect_venv(&context)?;
    venv_runner.run_and_die(cmd)
}

/// On Windows:
///   - same as run
/// On Linux:
///   - same as run, but create a new process instead of using execv()
// Note: mostly for tests. We want to *check* the return code of
// `dmenv run` and so we need a child process
pub fn run<T: AsRef<str>>(context: &Context, cmd: &[T]) -> Result<(), Error> {
    let Context { venv_runner, .. } = context;
    commands::expect_venv(&context)?;
    venv_runner.run(cmd)
}
