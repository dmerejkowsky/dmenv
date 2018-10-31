extern crate colored;
use colored::*;
extern crate dmenv;
extern crate structopt;
use dmenv::App;
use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(name = "dmenv", about = "The stupid virtualenv manager",)]
struct DmEnv {
    #[structopt(
        long = "env",
        help = "environment name",
        default_value = "default"
    )]
    env_name: String,

    #[structopt(subcommand)]
    cmd: Command,
}

#[derive(StructOpt)]
enum Command {
    #[structopt(name = "install", about = "Install all dependencies")]
    Install {
        #[structopt(long = "clean", help = "clean existing virtualenv",)]
        clean: bool,
        #[structopt(
            long = "upgrade-pip",
            help = "upgrade pip inside the virtualenv",
        )]
        upgrade_pip: bool,
    },

    #[structopt(name = "freeze", about = "(Re)-generate requirements.txt")]
    Freeze {},

    #[structopt(
        name = "run",
        about = "Run the given binary from the virtualenv"
    )]
    Run {
        #[structopt(name = "command")]
        cmd: Vec<String>,
    },
    #[structopt(name = "show", about = "Show path of the virtualenv")]
    Show {},
}

fn run_app() -> Result<(), dmenv::Error> {
    let opt = DmEnv::from_args();
    let app = App::new(&opt.env_name)?;
    match opt.cmd {
        Command::Install { clean, upgrade_pip } => {
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
        Command::Freeze {} => app.freeze(),
        Command::Run { cmd } => app.run(cmd),
        Command::Show {} => app.show(),
    }
}

fn main() {
    let result = run_app();
    if let Err(error) = result {
        eprintln!("{} {}", "Error".bold().red(), error);
        std::process::exit(1)
    };
}
