extern crate structopt;
use self::structopt::StructOpt;

pub struct TestApp {
    tmp_path: std::path::PathBuf,
}

///
/// An instance of dmenv::App designed for testing
///
/// By default, contains a copy of all the Python files in
/// demo/
impl TestApp {
    pub fn new(tmp_path: std::path::PathBuf) -> Self {
        if std::env::var("VIRTUAL_ENV").is_ok() {
            panic!("Please exit virtualenv before running tests");
        }
        let test_app = TestApp { tmp_path };
        test_app.copy_demo_files();
        test_app
    }

    fn copy_demo_files(&self) {
        std::fs::write(
            self.tmp_path.join("demo.py"),
            include_str!("../../demo/demo.py"),
        )
        .expect("");
        std::fs::write(
            self.tmp_path.join("test_demo.py"),
            include_str!("../../demo/test_demo.py"),
        )
        .expect("");
        std::fs::write(
            self.tmp_path.join("setup.py"),
            include_str!("../../demo/setup.py"),
        )
        .expect("");
    }

    pub fn remove_setup_py(&self) {
        self.remove_file("setup.py");
    }

    pub fn run(&self, args: Vec<String>) -> Result<(), dmenv::Error> {
        let mut cmd = vec![];
        cmd.extend(vec!["dmenv".to_string()]);
        let tmp_path: String = self.tmp_path.to_string_lossy().into();
        cmd.extend(vec!["--cwd".to_string(), tmp_path]);
        cmd.extend(args);
        let cmd = dmenv::Command::from_iter_safe(cmd).expect("");
        dmenv::run(cmd)
    }

    pub fn assert_run_ok(&self, args: &[&str]) {
        let args = to_string_args(&args);
        self.run(args).expect("");
    }

    pub fn read_lock(&self) -> String {
        let lock_path = &self.tmp_path.join(dmenv::LOCK_FILE_NAME);
        std::fs::read_to_string(lock_path).expect("")
    }

    pub fn assert_setup_py(&self) {
        self.assert_file("setup.py");
    }

    pub fn assert_file(&self, name: &str) {
        assert!(self.tmp_path.join(name).exists());
    }

    pub fn assert_run_error(&self, args: &[&str]) -> String {
        let args = to_string_args(&args);
        let res = self.run(args);
        res.unwrap_err().to_string()
    }

    pub fn write_lock(&self, contents: &str) {
        self.write_file(dmenv::LOCK_FILE_NAME, contents);
    }

    pub fn write_file(&self, name: &str, contents: &str) {
        let path = self.tmp_path.join(name);
        std::fs::write(path, &contents).expect("");
    }

    pub fn remove_file(&self, name: &str) {
        let path = self.tmp_path.join(name);
        std::fs::remove_file(path).expect("");
    }

    pub fn read_setup_py(&self) -> String {
        std::fs::read_to_string(self.tmp_path.join("setup.py")).expect("")
    }
}

pub fn to_string_args(args: &[&str]) -> Vec<String> {
    args.iter().map(|x| x.to_string()).collect()
}
