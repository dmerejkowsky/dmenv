use std::path::PathBuf;

mod cmd;
mod dependencies;
mod error;
#[cfg(unix)]
mod execv;
mod lock;
mod operations;
mod paths;
mod project;
mod python_info;
mod run;
mod settings;
#[cfg(windows)]
mod win_job;

pub use crate::cmd::Command;
use crate::cmd::SubCommand;
pub use crate::cmd::{print_error, print_info_1, print_info_2};
use crate::dependencies::FrozenDependency;
pub use crate::error::*;
use crate::lock::BumpType;
use crate::operations::{InitOptions, UpdateOptions};
use crate::paths::{Paths, PathsResolver};
pub use crate::paths::{DEV_LOCK_FILENAME, PROD_LOCK_FILENAME};
use crate::project::Metadata;
use crate::project::{PostInstallAction, ProcessScriptsMode, Project};
use crate::python_info::PythonInfo;
use crate::run::VenvRunner;
pub use crate::settings::Settings;

struct Context {
    paths: Paths,
    python_info: PythonInfo,
    settings: Settings,
    venv_runner: VenvRunner,
}

fn get_context(cmd: &Command) -> Result<Context, Error> {
    let project_path = if let Some(p) = &cmd.project_path {
        PathBuf::from(p)
    } else {
        look_up_for_project_path()?
    };
    let python_info = PythonInfo::new(&cmd.python_binary)?;
    let python_version = python_info.version.clone();
    let settings = Settings::from_shell(&cmd);
    let paths_resolver = PathsResolver::new(project_path.clone(), python_version, &settings);
    let paths = paths_resolver.paths()?;
    let venv_runner = VenvRunner::new(&project_path, &paths.venv);
    Ok(Context {
        paths,
        python_info,
        settings,
        venv_runner,
    })
}

pub fn run_cmd(cmd: Command) -> Result<(), Error> {
    if let SubCommand::Init {
        name,
        version,
        author,
        no_setup_cfg,
    } = &cmd.sub_cmd
    {
        return init(cmd.project_path, name, version, author, !no_setup_cfg);
    }

    let python_info = PythonInfo::new(&cmd.python_binary)?;

        PathBuf::from(p)
    } else {
        look_up_for_project_path()?
    };
    let settings = Settings::from_shell(&cmd);
    let project = Project::new(project_path, python_info, settings)?;

    let context = get_context(&cmd)?;

    match &cmd.sub_cmd {
        SubCommand::Install { no_develop } => {
            let post_install_action = if *no_develop {
                PostInstallAction::None
            } else {
                PostInstallAction::RunSetupPyDevelop
            };
            install(&context, post_install_action)
        }
        SubCommand::ProcessScripts { force } => {
            let mode = if *force {
                ProcessScriptsMode::Override
            } else {
                ProcessScriptsMode::Safe
            };
            process_scripts(&context, mode)
        }
        SubCommand::Clean {} => clean_venv(&context),
        SubCommand::Develop {} => develop(&context),
        SubCommand::Lock {
            python_version,
            sys_platform,
        } => {
            let update_options = UpdateOptions {
                python_version: python_version.clone(),
                sys_platform: sys_platform.clone(),
            };
            update_lock(&context, update_options)
        }
        SubCommand::BumpInLock { name, version, git } => {
            let bump_type = if *git {
                BumpType::Git
            } else {
                BumpType::Simple
            };
            bump_in_lock(&context, name, version, bump_type)
        }
        SubCommand::Run { ref cmd, no_exec } => {
            if *no_exec {
                run(&context, &cmd)
            } else {
                run_and_die(&context, &cmd)
            }
        }
        SubCommand::ShowDeps {} => show_deps(&context),
        SubCommand::ShowOutDated {} => project.show_outdated(),
        SubCommand::ShowVenvPath {} => project.show_venv_path(),
        SubCommand::ShowVenvBin {} => project.show_venv_bin_path(),

        SubCommand::Tidy {} => project.tidy(),

        SubCommand::UpgradePip {} => project.upgrade_pip(),
        _ => unimplemented!("Subcommand {:?} not handled", cmd.sub_cmd),
    }
}

fn init(
    project_path: Option<String>,
    name: &str,
    version: &str,
    author: &Option<String>,
    setup_cfg: bool,
) -> Result<(), Error> {
    let init_path = if let Some(p) = project_path {
        PathBuf::from(p)
    } else {
        std::env::current_dir().map_err(|e| Error::NoWorkingDirectory { io_error: e })?
    };

    let mut init_options = InitOptions::new(name.to_string(), version.to_string());
    if !setup_cfg {
        init_options.no_setup_cfg();
    };
    if let Some(author) = author {
        init_options.author(&author);
    }
    operations::init(&init_path, &init_options)
}

fn install(context: &Context, post_install_action: PostInstallAction) -> Result<(), Error> {
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

    ensure_venv(context)?;
    install_from_lock(context)?;

    match post_install_action {
        PostInstallAction::RunSetupPyDevelop => develop(context)?,
        PostInstallAction::None => (),
    }
    Ok(())
}

fn process_scripts(context: &Context, mode: ProcessScriptsMode) -> Result<(), Error> {
    operations::scripts::process(&context.paths, mode)
}

fn ensure_venv(context: &Context) -> Result<(), Error> {
    let Context { paths, .. } = context;
    if paths.venv.exists() {
        print_info_2(&format!(
            "Using existing virtualenv: {}",
            paths.venv.display()
        ));
    } else {
        create_venv(context)?;
    }
    Ok(())
}

fn create_venv(context: &Context) -> Result<(), Error> {
    let Context {
        paths,
        python_info,
        settings,
        ..
    } = context;
    // TODO: venv::create(context)
    operations::venv::create(&paths.venv, python_info, settings)
}

fn clean_venv(context: &Context) -> Result<(), Error> {
    let Context { paths, .. } = context;
    operations::venv::clean(paths.venv.clone())
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

/// (Re)generate the lock file
//
// Notes:
//
// * Abort if `setup.py` is not found
// * Create the virtualenv if required
// * Always upgrade pip :
//    * If that fails, we know if the virtualenv is broken
//    * Also, we know sure that `pip` can handle all the options
//      (such as `--local`, `--exclude-editable`) we use in the other functions
// * The path of the lock file is computed by PathsResolver.
//     See PathsResolver.paths() for details
fn update_lock(context: &Context, update_options: operations::UpdateOptions) -> Result<(), Error> {
    print_info_1("Updating lock");
    let Context { paths, .. } = context;
    if !&paths.setup_py.exists() {
        return Err(Error::MissingSetupPy {});
    }
    ensure_venv(&context)?;
    upgrade_pip(&context)?;
    install_editable(&context)?;
    let metadata = metadata(&context);
    let frozen_deps = get_frozen_deps(&context)?;
    let lock_path = &paths.lock;
    operations::lock::update(lock_path, frozen_deps, update_options, &metadata)
}

/// Bump a dependency in the lock file
fn bump_in_lock(
    context: &Context,
    name: &str,
    version: &str,
    bump_type: BumpType,
) -> Result<(), Error> {
    print_info_1(&format!("Bumping {} to {} ...", name, version));
    let metadata = metadata(&context);
    let Context { paths, .. } = context;
    operations::lock::bump(&paths.lock, name, version, bump_type, &metadata)
}

/// Runs `python setup.py` develop. Also called by `install` (unless InstallOptions.develop is false)
// Note: `lock()` will use `pip install --editable .` to achieve the same effect
fn develop(context: &Context) -> Result<(), Error> {
    let Context {
        paths, venv_runner, ..
    } = context;
    print_info_2("Running setup_py.py develop");
    if !&paths.setup_py.exists() {
        return Err(Error::MissingSetupPy {});
    }

    venv_runner.run(&["python", "setup.py", "develop", "--no-deps"])
}

/// Show the dependencies inside the virtualenv.
// Note: Run `pip list` so we get what's *actually* installed, not just
// the contents of the lock file
fn show_deps(context: &Context) -> Result<(), Error> {
    let Context { venv_runner, .. } = context;
    venv_runner.run(&["python", "-m", "pip", "list"])
}

fn upgrade_pip(context: &Context) -> Result<(), Error> {
    let Context { venv_runner, .. } = context;
    print_info_2("Upgrading pip");
    let cmd = &["python", "-m", "pip", "install", "pip", "--upgrade"];
    venv_runner.run(cmd).map_err(|_| Error::UpgradePipError {})
}

fn install_editable(context: &Context) -> Result<(), Error> {
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

fn get_install_editable_cmd(context: &Context) -> [&str; 6] {
    let Context { settings, .. } = context;
    let extra = if settings.production {
        ".[prod]"
    } else {
        ".[dev]"
    };
    ["python", "-m", "pip", "install", "--editable", extra]
}

fn metadata(context: &Context) -> Metadata {
    let Context { python_info, .. } = context;
    let dmenv_version = env!("CARGO_PKG_VERSION");
    let python_platform = &python_info.platform;
    let python_version = &python_info.version;
    Metadata {
        dmenv_version: dmenv_version.to_string(),
        python_platform: python_platform.to_string(),
        python_version: python_version.to_string(),
    }
}

fn look_up_for_project_path() -> Result<PathBuf, Error> {
    let mut candidate = std::env::current_dir()
        .map_err(|e| new_error(format!("Could not get current directory: {}", e)))?;
    loop {
        let setup_py_path = candidate.join("setup.py");
        if setup_py_path.exists() {
            return Ok(candidate);
        } else {
            let parent = candidate.parent();
            match parent {
                None => {
                    return Err(new_error(
                        "Could not find setup.py in any of the parent directories".to_string(),
                    ))
                }
                Some(p) => candidate = p.to_path_buf(),
            }
        }
    }
}

/// Get the list of the *actual* deps in the virtualenv by calling `pip freeze`.
fn get_frozen_deps(context: &Context) -> Result<Vec<FrozenDependency>, Error> {
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

/// Run a program from the virtualenv, making sure it dies
/// when we get killed and that the exit code is forwarded
fn run_and_die<T: AsRef<str>>(context: &Context, cmd: &[T]) -> Result<(), Error> {
    let Context { venv_runner, .. } = context;
    expect_venv(&context)?;
    venv_runner.run_and_die(cmd)
}

/// On Windows:
///   - same as run
/// On Linux:
///   - same as run, but create a new process instead of using execv()
// Note: mostly for tests. We want to *check* the return code of
// `dmenv run` and so we need a child process
fn run<T: AsRef<str>>(context: &Context, cmd: &[T]) -> Result<(), Error> {
    let Context { venv_runner, .. } = context;
    expect_venv(&context)?;
    venv_runner.run(cmd)
}

/// Make sure the virtualenv exists, or return an error
//
// Note: this must be called by any method that requires the
// virtualenv to exist, like `show_deps` or `run`:
// this ensures that error messages printed when the
// virtualenv does not exist are consistent.
fn expect_venv(context: &Context) -> Result<(), Error> {
    let Context { paths, .. } = context;
    operations::venv::expect(&paths.venv)
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_not_in_venv() {
        // If we run `cargo test` from an existing virtualenv, it will get
        // shared by all the tests and most of the integration tests will fail.
        //
        // The goal of _this_ test is to prevent integration tests from running
        // *at all* if we are inside a virtualenv. It works because `cargo test`
        // is clever and does not try to run integration tests when unit tests
        // fail.
        if std::env::var("VIRTUAL_ENV").is_ok() {
            panic!("Please exit virtualenv before running tests");
        }
    }
}
