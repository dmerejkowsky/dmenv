extern crate colored;
#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate structopt;

mod app;
mod config;
mod error;
mod options;

use app::App;
pub use app::LOCK_FILE_NAME;
pub use error::Error;
use options::Command;
pub use options::Options;

pub fn run_app(options: Options) -> Result<(), Error> {
    let app = App::new(
        &options.python_version,
        options.cfg_path,
        options.working_dir,
    )?;
    match options.cmd {
        Command::Install {} => app.install(),
        Command::Clean {} => app.clean(),
        Command::Init { name, version } => app.init(&name, &version),
        Command::Lock {} => app.lock(),
        Command::Run { cmd } => app.run(cmd),
        Command::Show {} => app.show(),
        Command::UpgradePip {} => app.upgrade_pip(),
    }
}
