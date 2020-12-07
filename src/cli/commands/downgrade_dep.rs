use crate::cli::commands;
use crate::error::*;
use crate::Context;

pub fn downgrade_dep(context: &Context, name: &str, version: &str) -> Result<(), Error> {
    commands::install_dep_then_lock("Downgrading", context, name, Some(version))
}
