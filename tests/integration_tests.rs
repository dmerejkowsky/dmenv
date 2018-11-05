extern crate structopt;

extern crate dmenv;
extern crate tempdir;

mod helpers;
use helpers::TestApp;

#[test]
fn show_does_not_crash() {
    let tmp_dir = tempdir::TempDir::new("test-dmenv").expect("");
    let test_app = TestApp::new(tmp_dir.path().to_path_buf());
    test_app.assert_run_ok(vec!["show"]);
}

#[test]
fn init_generates_setup_py() {
    let tmp_dir = tempdir::TempDir::new("test-dmenv").expect("");
    let test_app = TestApp::new(tmp_dir.path().to_path_buf());
    test_app.remove_setup_py();
    #[cfg_attr(rustfmt, rustfmt_skip)]
    test_app.assert_run_ok(vec![
        "init",
        "--name", "foo",
        "--version", "0.42",
        "--author", "jane@corp.com",
    ]);

    let written = test_app.read_setup_py();
    assert!(written.contains("foo"));
    assert!(written.contains("0.42"));
    assert!(written.contains("jane@corp.com"));
}

#[test]
fn init_does_not_overwrite_existing_setup_py() {
    let tmp_dir = tempdir::TempDir::new("test-dmenv").expect("");
    let test_app = TestApp::new(tmp_dir.path().to_path_buf());
    test_app.assert_run_error(vec!["init", "--name", "foo"]);
    test_app.assert_setup_py();
}

#[test]
fn lock_complains_if_setup_py_does_not_exist() {
    let tmp_dir = tempdir::TempDir::new("test-dmenv").expect("");
    let test_app = TestApp::new(tmp_dir.path().to_path_buf());
    test_app.remove_setup_py();
    test_app.assert_run_error(vec!["lock"]);
}

#[test]
fn lock_workflow() {
    let tmp_dir = tempdir::TempDir::new("test-dmenv").expect("");
    let test_app = TestApp::new(tmp_dir.path().to_path_buf());
    test_app.assert_run_ok(vec!["lock"]);
    test_app.assert_lock();
    test_app.assert_run_ok(vec!["run", "demo"]);
    test_app.assert_run_ok(vec!["run", "pytest"]);
}

#[test]
fn install_workflow() {
    let tmp_dir = tempdir::TempDir::new("test-dmenv").expect("");
    let test_app = TestApp::new(tmp_dir.path().to_path_buf());
    let lock_contents = include_str!("../demo/requirements.lock");
    test_app.write_lock(&lock_contents);
    test_app.assert_run_ok(vec!["install"]);
}

#[test]
fn run_without_args() {
    let tmp_dir = tempdir::TempDir::new("test-dmenv").expect("");
    let test_app = TestApp::new(tmp_dir.path().to_path_buf());
    test_app.assert_run_error(vec!["run"]);
}

#[test]
fn run_without_virtualenv() {
    let tmp_dir = tempdir::TempDir::new("test-dmenv").expect("");
    let test_app = TestApp::new(tmp_dir.path().to_path_buf());
    test_app.assert_run_error(vec!["run", "python"]);
}
