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

    #[structopt(
        name = "upgrade-pip",
        about = "Upgrade pip in the virtualenv"
    )]
    UpgradePip {},

    #[structopt(name = "show", about = "Show path of the virtualenv")]
    Show {},
}

fn run_app() -> Result<(), dmenv::Error> {
    let opt = DmEnv::from_args();
    let app = App::new(&opt.env_name)?;
    match opt.cmd {
        Command::Install { clean } => {
            if clean {
                app.clean()?
            }
            Ok(app.install()?)
        }
        Command::Freeze {} => app.freeze(),
        Command::Run { cmd } => app.run(cmd),
        Command::Show {} => app.show(),
        Command::UpgradePip {} => app.upgrade_pip(),
    }
}

fn main() {
    let result = run_app();
    if let Err(error) = result {
        eprintln!("{} {}", "Error".bold().red(), error);
        std::process::exit(1)
    };
}
