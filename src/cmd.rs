extern crate colored;
use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(name = "dmenv", about = "The stupid virtualenv manager")]
pub struct Command {
    #[structopt(long = "python", help = "python binary",)]
    pub python_binary: Option<String>,

    // Those are mainly useful for tests, but you never know:
    #[structopt(long = "cfg-path", help = "path to the config file")]
    pub cfg_path: Option<String>,

    #[structopt(
        long = "cwd",
        help = "path to use as the workining directory"
    )]
    pub working_dir: Option<String>,

    #[structopt(subcommand)]
    pub sub_cmd: SubCommand,
}

#[derive(StructOpt)]
pub enum SubCommand {
    #[structopt(name = "clean", about = "clean existing virtualenv")]
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

    #[structopt(name = "lock", about = "(Re)-generate requirements.lock")]
    Lock {},

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
