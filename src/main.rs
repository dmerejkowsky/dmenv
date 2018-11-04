extern crate colored;
use colored::*;
extern crate dmenv;
extern crate structopt;
use structopt::StructOpt;

fn main() {
    let cmd = dmenv::Command::from_args();
    let result = dmenv::run(cmd);
    if let Err(error) = result {
        eprintln!("{}: {}", "error".bold().red(), error);
        std::process::exit(1)
    };
}
