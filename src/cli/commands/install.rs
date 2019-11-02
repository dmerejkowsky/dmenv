use crate::commands;
use crate::error::*;
use crate::ui::*;
use crate::Context;
use crate::PostInstallAction;

pub fn install(context: &Context, post_install_action: PostInstallAction) -> Result<(), Error> {
    let Context {
        settings, paths, ..
    } = context;
    if settings.production {
        print_info_1("Preparing project for production")
    } else {
        print_info_1("Preparing project for development")
    };
    let lock_path = &paths.lock;
    if !lock_path.exists() {
        return Err(Error::MissingLock {
            expected_path: lock_path.to_path_buf(),
        });
    }

    commands::ensure_venv(context)?;
    install_from_lock(context)?;

    match post_install_action {
        PostInstallAction::RunSetupPyDevelop => commands::develop(context)?,
        PostInstallAction::None => (),
    }
    Ok(())
}

fn install_from_lock(context: &Context) -> Result<(), Error> {
    let Context {
        paths, venv_runner, ..
    } = context;
    let lock_path = &paths.lock;
    print_info_2(&format!(
        "Installing dependencies from {}",
        lock_path.display()
    ));
    // Since we'll be running the command using self.paths.project
    // as working directory, we must use the *relative* lock file
    // name when calling `pip install`.
    let lock_name = paths
        .lock
        .file_name()
        .unwrap_or_else(|| panic!("self.path.lock has no filename component"));

    let as_str = lock_name.to_string_lossy();
    let cmd = &["python", "-m", "pip", "install", "--requirement", &as_str];
    venv_runner.run(cmd)
}
