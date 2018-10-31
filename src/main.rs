extern crate colored;
use colored::*;
extern crate dmenv;
extern crate structopt;
use dmenv::App;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "dmenv",
    about = "The stupid virtualenv manager",
    author = ""   // hide the author
)]
enum DmEnv {
    #[structopt(name = "install", help = "install all dependencies")]
    Install {
        #[structopt(long = "clean", help = "clean existing virtualenv",)]
        clean: bool,
        #[structopt(
            long = "upgrade-pip",
            help = "upgrade pip inside the virtualenv",
        )]
        upgrade_pip: bool,
    },

    #[structopt(name = "freeze", help = "(re)-generate requirements.txt")]
    Freeze {},

    #[structopt(
        name = "run",
        help = "run the given binary from the virtualenv"
    )]
    Run {
        #[structopt(name = "command")]
        cmd: Vec<String>,
    },
}

fn run_app() -> Result<(), dmenv::Error> {
    let opt = DmEnv::from_args();
    let app = App::new()?;
    match opt {
        DmEnv::Install { clean, upgrade_pip } => {
            if clean {
                app.clean()?;
            }
            app.install()?;
            if upgrade_pip {
                app.upgrade_pip()
            } else {
                Ok(())
            }
        }
        DmEnv::Freeze {} => app.freeze(),
        DmEnv::Run { cmd } => app.run(cmd),
    }
}

fn main() {
    let result = run_app();
    if let Err(error) = result {
        eprintln!("{} {}", "Error".bold().red(), error);
        std::process::exit(1)
    };
}
