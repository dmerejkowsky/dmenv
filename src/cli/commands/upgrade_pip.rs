use crate::error::*;
use crate::ui::*;
use crate::Context;

pub fn upgrade_pip(context: &Context) -> Result<(), Error> {
    let Context { venv_runner, .. } = context;
    print_info_2("Upgrading pip");
    let cmd = &["python", "-m", "pip", "install", "pip", "--upgrade"];
    venv_runner.run(cmd).map_err(|_| Error::UpgradePipError {})
}
