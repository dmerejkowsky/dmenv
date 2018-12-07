extern crate structopt;

extern crate dmenv;
extern crate tempdir;

mod helpers;
use helpers::TestApp;

#[test]
fn show_venv_path() {
    let tmp_dir = tempdir::TempDir::new("test-dmenv").expect("");
    let test_app = TestApp::new(tmp_dir.path().to_path_buf());
    test_app.assert_run_ok(vec!["show:venv_path"]);
}

#[test]
fn init_generates_setup_cfg() {
    let tmp_dir = tempdir::TempDir::new("test-dmenv").expect("");
    let test_app = TestApp::new(tmp_dir.path().to_path_buf());
    test_app.remove_setup_py();
    test_app.remove_setup_cfg();
    #[cfg_attr(rustfmt, rustfmt_skip)]
    test_app.assert_run_ok(vec![
        "init", "foo",
        "--version", "0.42",
        "--author", "jane@corp.com",
    ]);

    let written = test_app.read_setup_cfg();
    assert!(written.contains("foo"));
    assert!(written.contains("0.42"));
    assert!(written.contains("jane@corp.com"));
}

#[test]
fn bump_in_lock_simple() {
    let tmp_dir = tempdir::TempDir::new("test-dmenv").expect("");
    let test_app = TestApp::new(tmp_dir.path().to_path_buf());
    let lock_contents = r#"
foo==0.42
-e git+ssh://git@gitlab.local/bar@abc42f#egg=bar
"#;
    test_app.write_lock(&lock_contents);

    test_app.assert_run_ok(vec!["bump-in-lock", "foo", "0.43"]);

    let actual_contents = test_app.read_lock();
    let expected_contents = lock_contents.replace("0.42", "0.43");
    assert_eq!(actual_contents, expected_contents);
}

#[test]
fn bump_in_lock_git() {
    let tmp_dir = tempdir::TempDir::new("test-dmenv").expect("");
    let test_app = TestApp::new(tmp_dir.path().to_path_buf());
    let lock_contents = r#"
foo==0.42
-e git+ssh://git@gitlab.local/bar@abc42f#egg=bar
"#;
    test_app.write_lock(&lock_contents);

    test_app.assert_run_ok(vec!["bump-in-lock", "--git", "bar", "bfc42a"]);
}

#[test]
fn init_does_not_overwrite_existing_setup_py() {
    let tmp_dir = tempdir::TempDir::new("test-dmenv").expect("");
    let test_app = TestApp::new(tmp_dir.path().to_path_buf());
    test_app.assert_run_error(vec!["init", "foo"]);
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
    let lock_contents = test_app.read_lock();
    assert!(lock_contents.contains("pytest=="));
    assert!(!lock_contents.contains("pkg-resources=="));
    test_app.assert_run_ok(vec!["show:deps"]);
    test_app.assert_run_ok(vec!["run", "demo"]);
    test_app.assert_run_ok(vec!["run", "pytest"]);
}

#[test]
fn install_workflow_all_in_one() {
    let tmp_dir = tempdir::TempDir::new("test-dmenv").expect("");
    let test_app = TestApp::new(tmp_dir.path().to_path_buf());
    let lock_contents = include_str!("../demo/requirements.lock");
    test_app.write_lock(&lock_contents);
    test_app.assert_run_ok(vec!["install"]);
}

#[test]
fn install_workflow_step_by_step() {
    let tmp_dir = tempdir::TempDir::new("test-dmenv").expect("");
    let test_app = TestApp::new(tmp_dir.path().to_path_buf());
    let lock_contents = include_str!("../demo/requirements.lock");
    test_app.write_lock(&lock_contents);
    test_app.assert_run_ok(vec!["install", "--no-develop", "--no-upgrade-pip"]);
    test_app.assert_run_ok(vec!["develop"]);
    test_app.assert_run_ok(vec!["run", "demo"]);
}

#[test]
fn install_without_lock() {
    let tmp_dir = tempdir::TempDir::new("test-dmenv").expect("");
    let test_app = TestApp::new(tmp_dir.path().to_path_buf());
    test_app.assert_run_error(vec!["install"]);
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
