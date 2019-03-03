use structopt::StructOpt;

fn main() {
    let cmd = dmenv::Command::from_args();
    let mut settings = dmenv::Settings::from_env();
    // TODO: settings.apply_cmd_flags(cmd)
    settings.system_site_packages = cmd.system_site_packages;
    settings.production = cmd.production;
    let result = dmenv::run(cmd, settings);
    if let Err(error) = result {
        dmenv::print_error(&error.to_string());
        std::process::exit(1)
    };
}
