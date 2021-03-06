use structopt::StructOpt;

fn main() {
    let cmd = dmenv::Command::from_args();
    let result = dmenv::run_cmd(cmd);
    if let Err(error) = result {
        dmenv::print_error(&error.to_string());
        std::process::exit(1)
    };
}
