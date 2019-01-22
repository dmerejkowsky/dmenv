mod helpers;
use crate::helpers::TestApp;

#[test]
fn show_venv_path() {
    let test_app = TestApp::new();
    test_app.assert_run_ok(&["show:venv_path"]);
}

#[test]
fn init_generates_setup_py() {
    let test_app = TestApp::new();
    test_app.remove_setup_py();
    #[rustfmt::skip]
    test_app.assert_run_ok(&[
        "init", "foo",
        "--version", "0.42",
        "--author", "jane@corp.com",
    ]);

    let written = test_app.read_setup_py();
    assert!(written.contains("foo"));
    assert!(written.contains("0.42"));
    assert!(written.contains("jane@corp.com"));
}

#[test]
fn bump_in_lock_simple() {
    let test_app = TestApp::new();
    let lock_contents = "bar==1.3\nfoo==0.42\n";
    test_app.write_lock(&lock_contents);

    test_app.assert_run_ok(&["bump-in-lock", "foo", "0.43"]);
    let actual_contents = test_app.read_lock();
    let expected_contents = lock_contents.replace("0.42", "0.43");
    assert_eq!(actual_contents, expected_contents);
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
    let lock_contents = test_app.read_lock();
    assert!(lock_contents.contains("pytest=="));
    assert!(!lock_contents.contains("pkg-resources=="));
    test_app.assert_run_ok(&["show:deps"]);
    test_app.assert_run_ok(&["run", "--no-exec", "demo"]);
    test_app.assert_run_ok(&["run", "--no-exec", "pytest"]);
}

#[test]
fn install_workflow_all_in_one() {
    let test_app = TestApp::new();
    let lock_contents = include_str!("../demo/requirements.lock");
    test_app.write_lock(&lock_contents);
    test_app.assert_run_ok(&["install"]);
}

#[test]
fn install_workflow_step_by_step() {
    let test_app = TestApp::new();
    let lock_contents = include_str!("../demo/requirements.lock");
    test_app.write_lock(&lock_contents);
    test_app.assert_run_ok(&["install", "--no-develop", "--no-upgrade-pip"]);
    test_app.assert_run_ok(&["develop"]);
    test_app.assert_run_ok(&["run", "--no-exec", "demo"]);
}

#[test]
fn install_without_lock() {
    let test_app = TestApp::new();
    test_app.remove_lock();
    test_app.assert_run_error(&["install"]);
}

#[test]
fn run_without_args() {
    let test_app = TestApp::new();
    test_app.assert_run_error(&["run"]);
}

#[test]
fn run_without_virtualenv() {
    let test_app = TestApp::new();
    test_app.assert_run_error(&["run", "python"]);
}
