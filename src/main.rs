extern crate colored;
extern crate dmenv;
extern crate structopt;
use structopt::StructOpt;

fn main() {
    let cmd = dmenv::Command::from_args();
    let result = dmenv::run(cmd);
    if let Err(error) = result {
        dmenv::print_error(&error.to_string());
        std::process::exit(1)
    };
}
