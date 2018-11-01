extern crate colored;
use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(name = "dmenv", about = "The stupid virtualenv manager",)]
pub struct Options {
    #[structopt(
        long = "env",
        help = "environment name",
        default_value = "default"
    )]
    pub env_name: String,

    #[structopt(subcommand)]
    pub cmd: Command,
}

#[derive(StructOpt)]
pub enum Command {
    #[structopt(name = "clean", about = "clean existing virtualenv",)]
    Clean {},

    #[structopt(name = "install", about = "Install all dependencies")]
    Install {},

    #[structopt(name = "init", about = "Initialize a new project")]
    Init {
        #[structopt(long = "name", help = "Project name")]
        name: String,
        #[structopt(
            long = "version",
            help = "Project version",
            default_value = "0.1.0"
        )]
        version: String,
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
