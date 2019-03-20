use crate::cmd::Command;

#[derive(Debug, Clone)]
/// Represent variables that change behavior of
/// the VenvManager or PathsResolver structs.
pub struct Settings {
    pub venv_from_stdlib: bool,
    pub venv_outside_project: bool,
    pub production: bool,
}

impl Default for Settings {
    fn default() -> Settings {
        Settings {
            venv_from_stdlib: true,
            venv_outside_project: false,
            production: false,
        }
    }
}

impl Settings {
    /// Construct a new Settings instance using
    /// options from the command line (the `cmd` parameter)
    /// and environment variables.
    //
    // Note:  Called in `main()` and in test heplers.
    pub fn from_shell(cmd: &Command) -> Settings {
        let mut res = Settings {
            production: cmd.production,
            ..Default::default()
        };
        if std::env::var("DMENV_NO_VENV_STDLIB").is_ok() {
            res.venv_from_stdlib = false;
        }
        if std::env::var("DMENV_VENV_OUTSIDE_PROJECT").is_ok() {
            res.venv_outside_project = true;
        }
        res
    }
}
