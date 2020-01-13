use crate::cli::commands;
use crate::error::*;
use crate::ui::*;
use crate::Context;

/// Runs `python setup.py` develop. Also called by `install` (unless InstallOptions.develop is false)
// Note: `lock()` will use `pip install --editable .` to achieve the same effect
pub fn develop(context: &Context) -> Result<(), Error> {
    let Context {
        paths, venv_runner, ..
    } = context;
    commands::expect_venv(&context)?;
    print_info_2("Running setup_py.py develop");
    if !&paths.setup_py.exists() {
        return Err(Error::MissingSetupPy {});
    }

    venv_runner.run(&["python", "setup.py", "develop", "--no-deps"])
}
