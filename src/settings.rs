#[derive(Debug)]
pub struct Settings {
    pub venv_from_stdlib: bool,
    pub venv_outside_project: bool,
}

impl Default for Settings {
    fn default() -> Settings {
        Settings {
            venv_from_stdlib: true,
            venv_outside_project: false,
        }
    }
}

impl Settings {
    pub fn from_env() -> Settings {
        let mut res = Settings::default();
        if std::env::var("DMENV_NO_VENV_STDLIB").is_ok() {
            res.venv_from_stdlib = false;
        }
        if std::env::var("DMENV_VENV_OUTSIDE_PROJECT").is_ok() {
            res.venv_outside_project = true;
        }
        res
    }
}
