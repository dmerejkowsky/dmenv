extern crate structopt;

extern crate dmenv;
extern crate tempdir;

mod helpers;
use helpers::TestApp;

#[test]
fn show_complains_if_cfg_is_missing() {
    let tmp_dir = tempdir::TempDir::new("test-dmenv").expect("");
    let test_app = TestApp::new(tmp_dir.path().to_path_buf());
    test_app.remove_cfg();
    test_app.assert_run_error(vec!["show"]);
}

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
    test_app.assert_run_ok(vec!["init", "--name", "foo"]);
    test_app.assert_setup_py();
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
