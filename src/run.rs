use std::path::{Path, PathBuf};

use colored::*;

use crate::error::*;
#[cfg(unix)]
use crate::execv::execv;
#[cfg(windows)]
use crate::win_job;

use crate::paths::SCRIPTS_SUBDIR;

pub struct VenvRunner {
    project_path: PathBuf,
    venv_path: PathBuf,
}

#[derive(Debug)]
struct RunnableCommand {
    binary_path: PathBuf,
    args: Vec<String>,
}

impl RunnableCommand {
    pub fn new<T: AsRef<str>>(binary_path: &Path, args: &[T]) -> Result<Self, Error> {
        if !binary_path.exists() {
            return Err(Error::Other {
                message: format!("Cannot run: {} does not exist", binary_path.display()),
            });
        }
        let res = RunnableCommand {
            binary_path: binary_path.to_path_buf(),
            args: args.iter().map(|x| x.as_ref().to_string()).collect(),
        };
        Ok(res)
    }

    pub fn print_self(&self) {
        println!(
            "{} {} {}",
            "$".blue(),
            self.binary_path.display(),
            self.args.join(" ")
        );
    }
}

impl VenvRunner {
    pub fn new(project_path: &Path, venv_path: &Path) -> Self {
        VenvRunner {
            project_path: project_path.to_path_buf(),
            venv_path: venv_path.to_path_buf(),
        }
    }

    pub fn run_and_die<T: AsRef<str>>(&self, cmd: &[T]) -> Result<(), Error> {
        #[cfg(windows)]
        {
            unsafe {
                win_job::setup();
            }
            self.run(cmd)
        }

        #[cfg(unix)]
        {
            let runnable = self.get_runnable(cmd)?;
            runnable.print_self();
            let mut cmd: Vec<&str> = runnable.args.iter().map(AsRef::as_ref).collect();
            let arg0 = &runnable.binary_path;
            let arg0 = arg0
                .to_str()
                .ok_or_else(|| new_error(&format!("Could not convert {:?} to string", arg0)))?;
            cmd.insert(0, &arg0);
            execv(arg0, &cmd)
        }
    }

    pub fn run<T: AsRef<str>>(&self, cmd: &[T]) -> Result<(), Error> {
        let runnable = self.get_runnable(cmd)?;
        runnable.print_self();
        run(&self.project_path, &runnable.binary_path, &runnable.args)
    }

    pub fn get_output<T: AsRef<str>>(&self, cmd: &[T]) -> Result<String, Error> {
        let runnable = self.get_runnable(cmd)?;
        get_output(&self.project_path, &runnable.binary_path, &runnable.args)
    }

    fn get_runnable<T: AsRef<str>>(&self, cmd: &[T]) -> Result<RunnableCommand, Error> {
        let first_arg = &cmd[0].as_ref();
        if first_arg.ends_with(".py") {
            let script_path = self.project_path.join(first_arg);
            if script_path.exists() {
                let python_binary = self.get_binary_path("python");
                return RunnableCommand::new(&python_binary, &cmd);
            }
        }

        let binary_path = self.get_binary_path(&cmd[0].as_ref());
        let args = &cmd[1..];
        RunnableCommand::new(&binary_path, args)
    }

    pub fn binaries_path(&self) -> PathBuf {
        self.venv_path.join(SCRIPTS_SUBDIR)
    }

    fn get_binary_path(&self, name: &str) -> PathBuf {
        let binary_name = Self::get_binary_name(name);
        self.binaries_path().join(&binary_name)
    }

    fn get_binary_name(name: &str) -> String {
        #[cfg(windows)]
        let suffix = ".exe";
        #[cfg(unix)]
        let suffix = "";
        format!("{}{}", name, suffix)
    }
}

pub fn run<T: AsRef<str>>(
    working_path: &Path,
    binary_path: &Path,
    args: &[T],
) -> Result<(), Error> {
    let args: Vec<&str> = args.iter().map(AsRef::as_ref).collect();
    let command = std::process::Command::new(binary_path)
        .args(args)
        .current_dir(working_path)
        .status();
    let command = command.map_err(|e| Error::ProcessWaitError { io_error: e })?;
    if !command.success() {
        return Err(new_error("command failed"));
    }
    Ok(())
}

fn get_output<T: AsRef<str>>(
    working_path: &Path,
    binary_path: &Path,
    args: &[T],
) -> Result<String, Error> {
    let args: Vec<&str> = args.iter().map(AsRef::as_ref).collect();
    let cmd_str = format!("{} {}", binary_path.display(), args.join(" "));
    let command = std::process::Command::new(binary_path)
        .args(args)
        .current_dir(working_path)
        .output();

    let command = command.map_err(|e| Error::ProcessOutError { io_error: e })?;
    if !command.status.success() {
        return Err(new_error(&format!(
            "`{}` failed\n: {}",
            cmd_str,
            String::from_utf8_lossy(&command.stderr)
        )));
    }
    Ok(String::from_utf8_lossy(&command.stdout).to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    struct FileSystem {
        // This struct simply holds the tempdir so that the temp dir is
        // removed when it is droped
        #[allow(dead_code)]
        tmp_dir: tempdir::TempDir,
        project: PathBuf,
        venv: PathBuf,
    }

    impl FileSystem {
        fn new() -> Self {
            let tmp_dir = tempdir::TempDir::new("test-dmenv").unwrap();
            let tmp_path = tmp_dir.path();
            let project_path = tmp_path.join("project");
            let venv_path = tmp_path.join("venv");

            std::fs::create_dir_all(&project_path).unwrap();
            std::fs::create_dir_all(&venv_path).unwrap();

            let res = FileSystem {
                tmp_dir,
                project: project_path.to_path_buf(),
                venv: venv_path.to_path_buf(),
            };
            res.init_venv();
            res
        }

        fn add_script_in_project(&self, name: &str) -> PathBuf {
            let res = &self.project.join(name);
            std::fs::write(res, "").unwrap();
            res.to_path_buf()
        }

        fn add_binary_in_venv(&self, name: &str) -> PathBuf {
            let binary_name = VenvRunner::get_binary_name(name);
            let res = self.venv.join(SCRIPTS_SUBDIR).join(binary_name);
            std::fs::write(&res, "").unwrap();
            res
        }

        fn init_venv(&self) {
            let scripts_dir = self.venv.join(SCRIPTS_SUBDIR);
            std::fs::create_dir_all(scripts_dir).unwrap();
            self.add_binary_in_venv("python");
        }
    }

    impl RunnableCommand {
        fn assert_binary(&self, path: &Path) {
            assert_eq!(&self.binary_path, path);
        }

        fn assert_args(&self, args: &[&str]) {
            let expected_args: Vec<String> = args.iter().map(|x| x.to_string()).collect();
            assert_eq!(self.args, expected_args);
        }
    }

    #[test]
    fn test_run_script_in_project() {
        let fs = FileSystem::new();
        fs.add_script_in_project("foo.py");
        let venv_runner = VenvRunner::new(&fs.project, &fs.venv);
        let runnable = venv_runner.get_runnable(&["foo.py"]).unwrap();
        let expected_binary = venv_runner.get_binary_path("python");
        runnable.assert_binary(&expected_binary);
        runnable.assert_args(&["foo.py"]);
    }

    #[test]
    fn test_run_python() {
        let fs = FileSystem::new();
        let venv_runner = VenvRunner::new(&fs.project, &fs.venv);
        let runnable = venv_runner.get_runnable(&["python", "foo.py"]).unwrap();
        let expected_binary = venv_runner.get_binary_path("python");
        runnable.assert_binary(&expected_binary);
        runnable.assert_args(&["foo.py"]);
    }

    #[test]
    fn test_run_py_script_in_venv() {
        let fs = FileSystem::new();
        let docutils_script = fs.add_binary_in_venv("rst2html.py");
        let venv_runner = VenvRunner::new(&fs.project, &fs.venv);
        let runnable = venv_runner.get_runnable(&["rst2html.py"]).unwrap();
        runnable.assert_binary(&docutils_script);
        runnable.assert_args(&[]);
    }
}
