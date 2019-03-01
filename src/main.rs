use structopt::StructOpt;

fn main() {
    let cmd = dmenv::Command::from_args();
    let settings = dmenv::Settings::from_env();
    let result = dmenv::run(cmd, settings);
    if let Err(error) = result {
        dmenv::print_error(&error.to_string());
        std::process::exit(1)
    };
}
