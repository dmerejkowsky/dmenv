extern crate structopt;
use structopt::StructOpt;

extern crate dmenv;
extern crate tempdir;

#[test]
fn test_init_complains_if_cfg_path_does_not_exist() {
    let options =
        dmenv::Options::from_iter_safe(["dmenv", "--cfg-path", "nosuch.toml", "show"].iter())
            .expect("");
    let result = dmenv::run_app(options);
    assert!(result.is_err());
}

struct TestApp {
    tmp_dir: tempdir::TempDir,
}

impl TestApp {
    fn new(tmp_dir: tempdir::TempDir) -> Self {
        println!("Running in {}", tmp_dir.path().to_string_lossy());
        let test_app = TestApp { tmp_dir };
        test_app.ensure_cfg();
        test_app
    }

    fn ensure_cfg(&self) {
        let test_toml_path = &self.tmp_dir.path().join("test.toml");
        std::fs::write(
            test_toml_path,
            r#"
[env.default]
python = "/usr/bin/python"
"#,
        ).expect("");
    }

    fn run(&self, args: Vec<String>) -> Result<(), dmenv::Error> {
        let mut cmd = vec![];
        cmd.extend(vec!["dmenv".to_string()]);
        let tmp_path: String = self.tmp_dir.path().to_string_lossy().into();
        cmd.extend(vec!["--cwd".to_string(), tmp_path]);

        let cfg_path: String = self
            .tmp_dir
            .path()
            .join("test.toml")
            .to_string_lossy()
            .into();
        cmd.extend(vec!["--cfg-path".to_string(), cfg_path]);
        cmd.extend(args);
        let options = dmenv::Options::from_iter_safe(cmd).expect("");
        dmenv::run_app(options)
    }
}

fn to_string_args(args: Vec<&str>) -> Vec<String> {
    args.iter().map(|x| x.to_string()).collect()
}

#[test]
fn test_init_generates_a_setup_py() {
    let tmp_dir = tempdir::TempDir::new("test-dmenv").expect("");
    let test_app = TestApp::new(tmp_dir);
    test_app
        .run(to_string_args(vec!["init", "--name", "foo"]))
        .expect("")
}
