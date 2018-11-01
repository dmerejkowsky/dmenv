extern crate colored;
use colored::*;
extern crate dmenv;
extern crate structopt;
use structopt::StructOpt;

fn main() {
    let options = dmenv::Options::from_args();
    let result = dmenv::run_app(options);
    if let Err(error) = result {
        eprintln!("{}: {}", "Error".bold().red(), error);
        std::process::exit(1)
    };
}
