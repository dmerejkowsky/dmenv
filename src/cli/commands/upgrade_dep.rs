use crate::cli::commands;
use crate::error::*;
use crate::Context;

pub fn upgrade_dep(context: &Context, name: &str, version: Option<&str>) -> Result<(), Error> {
    commands::install_dep_then_lock("Upgrading", context, name, version)
}
