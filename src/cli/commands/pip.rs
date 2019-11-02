use crate::dependencies::FrozenDependency;
use crate::error::*;
use crate::ui::*;
use crate::Context;

pub fn upgrade_pip(context: &Context) -> Result<(), Error> {
    let Context { venv_runner, .. } = context;
    print_info_2("Upgrading pip");
    let cmd = &["python", "-m", "pip", "install", "pip", "--upgrade"];
    venv_runner.run(cmd).map_err(|_| Error::UpgradePipError {})
}

pub fn install_editable(context: &Context) -> Result<(), Error> {
    let Context {
        settings,
        venv_runner,
        ..
    } = context;
    let mut message = "Installing deps from setup.py".to_string();
    if settings.production {
        message.push_str(" using 'prod' extra dependencies");
    } else {
        message.push_str(" using 'dev' extra dependencies");
    }
    print_info_2(&message);
    let cmd = get_install_editable_cmd(&context);
    venv_runner.run(&cmd)
}

pub fn install_editable_with_constraint(context: &Context) -> Result<(), Error> {
    let Context {
        paths, venv_runner, ..
    } = context;
    let lock_path = &paths.lock;
    let message = format!(
        "Installing deps from setup.py, constrained by {}",
        lock_path.display()
    );
    print_info_2(&message);
    let lock_path_str = lock_path.to_string_lossy();
    let mut cmd = get_install_editable_cmd(&context).to_vec();
    cmd.extend(&["--constraint", &lock_path_str]);
    venv_runner.run(&cmd)
}

fn get_install_editable_cmd(context: &Context) -> [&str; 6] {
    let Context { settings, .. } = context;
    let extra = if settings.production {
        ".[prod]"
    } else {
        ".[dev]"
    };
    ["python", "-m", "pip", "install", "--editable", extra]
}

/// Get the list of the *actual* deps in the virtualenv by calling `pip freeze`.
pub fn get_frozen_deps(context: &Context) -> Result<Vec<FrozenDependency>, Error> {
    let freeze_output = run_pip_freeze(&context)?;
    // First, collect all the `pip freeze` lines into frozen dependencies
    let deps: Result<Vec<_>, _> = freeze_output
        .lines()
        .map(|x| FrozenDependency::from_string(x.into()))
        .collect();
    let deps = deps?;
    // Then filter out pkg-resources: this works around a Debian bug in pip:
    // https://bugs.debian.org/cgi-bin/bugreport.cgi?bug=871790
    let res: Vec<_> = deps
        .into_iter()
        .filter(|x| x.name != "pkg-resources")
        .collect();
    Ok(res)
}

fn run_pip_freeze(context: &Context) -> Result<String, Error> {
    let Context { venv_runner, .. } = context;
    #[rustfmt::skip]
        let cmd = &[
            "python", "-m", "pip", "freeze",
            "--exclude-editable",
            "--all",
            "--local",
        ];
    venv_runner.get_output(cmd)
}
