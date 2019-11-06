mod helpers;
use crate::helpers::TestApp;

#[test]
fn show_venv_path() {
    let test_app = TestApp::new();
    test_app.assert_run_ok(&["show:venv_path"]);
}

#[test]
fn init_works() {
    let test_app = TestApp::new();
    test_app.remove_setup_py();
    test_app.remove_setup_cfg();

    #[rustfmt::skip]
    test_app.assert_run_ok(&[
        "init", "foo",
        "--version", "0.42",
        "--author", "jane@corp.com",
    ]);

    test_app.assert_run_ok(&["lock"]);
}

#[test]
fn bump_in_lock_simple() {
    let test_app = TestApp::new();
    test_app.assert_run_ok(&["bump-in-lock", "attrs", "19.2.0"]);
    let actual = test_app.read_dev_lock();
    assert!(actual.contains("attrs==19.2.0"));
}

#[test]
fn init_does_not_overwrite_existing_setup_py() {
    let test_app = TestApp::new();
    test_app.assert_run_error(&["init", "foo"]);
    test_app.assert_setup_py();
}

#[test]
fn lock_complains_if_setup_py_does_not_exist() {
    let test_app = TestApp::new();
    test_app.remove_setup_py();
    test_app.assert_run_error(&["lock"]);
}

#[test]
fn lock_workflow() {
    let test_app = TestApp::new();
    test_app.assert_run_ok(&["lock"]);
    let lock_contents = test_app.read_dev_lock();
    assert!(lock_contents.contains("pytest=="));
    assert!(!lock_contents.contains("pkg-resources=="));
    test_app.assert_run_ok(&["show:deps"]);
    test_app.assert_run_ok(&["show:outdated"]);
    test_app.assert_run_ok(&["run", "--no-exec", "demo"]);
    test_app.assert_run_ok(&["run", "--no-exec", "pytest"]);
}

#[test]
fn production_workflow() {
    let test_app = TestApp::new();
    test_app.remove_prod_lock();
    test_app.assert_run_ok(&["--production", "lock"]);
    let lock_contents = test_app.read_prod_lock();
    assert!(!lock_contents.contains("pytest"));
    assert!(lock_contents.contains("path.py"));
    assert!(lock_contents.contains("gunicorn"));

    test_app.assert_run_ok(&["--production", "clean"]);
    test_app.assert_run_ok(&["--production", "install"]);
    test_app.assert_run_ok(&["--production", "run", "--no-exec", "demo"]);
}

#[test]
fn install_workflow_basic() {
    let test_app = TestApp::new();
    test_app.assert_run_ok(&["install"]);
    test_app.assert_run_ok(&["run", "--no-exec", "demo"]);
    test_app.assert_run_ok(&["run", "--no-exec", "pytest"]);
}

#[test]
fn install_workflow_step_by_step() {
    let test_app = TestApp::new();
    test_app.assert_run_ok(&["install", "--no-develop"]);
    test_app.assert_run_ok(&["develop"]);
    test_app.assert_run_ok(&["run", "--no-exec", "demo"]);
    test_app.assert_run_ok(&["run", "--no-exec", "pytest"]);
}

#[test]
fn run_project_script() {
    let test_app = TestApp::new();
    test_app.assert_run_ok(&["install"]);
    test_app.assert_run_ok(&["run", "--no-exec", "demo.py"]);
}

#[test]
fn install_without_lock() {
    let test_app = TestApp::new();
    test_app.remove_dev_lock();
    test_app.assert_run_error(&["install"]);
}

#[test]
fn run_without_virtualenv() {
    let test_app = TestApp::new();
    test_app.assert_run_error(&["run", "python"]);
}

#[test]
fn test_process_scripts() {
    let test_app = TestApp::new();
    let scripts_path = test_app.path().join("scripts");
    std::fs::create_dir_all(&scripts_path).unwrap();
    std::env::set_var("DMENV_SCRIPTS_PATH", &scripts_path);
    test_app.assert_run_ok(&["install"]);
    test_app.assert_run_ok(&["process-scripts"]);
    #[cfg(unix)]
    let script_path = scripts_path.join("demo");
    #[cfg(windows)]
    let script_path = scripts_path.join("demo.exe");
    assert!(script_path.exists());
    let command = std::process::Command::new(script_path).status().unwrap();
    assert!(command.success())
}

#[test]
fn test_tidy() {
    let test_app = TestApp::new();
    // Setup is:
    //  * `attrs` (outdated) and `appdirs` in the lock
    //  * the `setup.cfg` file only contains 'attrs'
    //
    test_app.override_lock(include_str!("tidy/requirements.lock"));
    test_app.override_setup_cfg(include_str!("tidy/setup.cfg"));

    // Run tidy
    test_app.assert_run_ok(&["tidy"]);

    // Running `dmenv lock --clean` should remove `appdirs` from the lock
    // but *not* bump `attrs`
    let lock_contents = test_app.read_dev_lock();
    assert!(!lock_contents.contains("appdirs"));
    assert!(lock_contents.contains("attrs==19.2.0"));
}
