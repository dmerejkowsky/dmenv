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
    #[derive(Debug, StructOpt)]
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

    #[derive(Debug, StructOpt)]
    #[structopt(name = "freeze", help = "(re)-generate requirements.txt")]
    Freeze {},
}

fn main() {
    let opt = DmEnv::from_args();
    let app = App::new();
    match opt {
        DmEnv::Install { clean, upgrade_pip } => {
            if clean {
                app.clean()
            }
            app.install();
            if upgrade_pip {
                app.upgrade_pip()
            }
        }
        DmEnv::Freeze {} => app.freeze(),
    }
}
