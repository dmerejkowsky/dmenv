use crate::error::*;
use crate::operations;
use crate::Context;
use crate::ProcessScriptsMode;

pub fn process_scripts(context: &Context, mode: ProcessScriptsMode) -> Result<(), Error> {
    operations::scripts::process(&context.paths, mode)
}
