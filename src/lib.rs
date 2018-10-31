extern crate appdirs;

pub struct App {
    venv_path: std::path::PathBuf,
    requirements_lock_path: std::path::PathBuf,
}

impl App {
    pub fn new() -> Self {
        let current_dir = std::env::current_dir();
        let current_dir = current_dir.unwrap();
        let name = current_dir.file_name().unwrap();
        let data_dir = appdirs::user_data_dir(Some("dmenv"), None, false);
        let data_dir = data_dir.unwrap();
        let venv_path = data_dir.join(name);
        let requirements_lock_path = current_dir.join("requirements.lock");
        App {
            venv_path,
            requirements_lock_path,
        }
    }

    pub fn clean(&self) {
        println!("Cleaning {}", &self.venv_path.to_string_lossy());
        if !self.venv_path.exists() {
            return;
        }
        std::fs::remove_dir_all(&self.venv_path).unwrap();
    }

    pub fn install(&self) {
        if !self.venv_path.exists() {
            self.create_venv()
        }

        if !self.requirements_lock_path.exists() {
            eprintln!(
                "{} does not exist. Please run dmenv freeze",
                &self.requirements_lock_path.to_string_lossy()
            );
            return;
        }

        self.run_pip_install();
    }

    pub fn freeze(&self) {
        println!("Generating requirements.txt from setup.py");

        self.install_editable();
        self.run_pip_freeze();
    }

    fn create_venv(&self) {
        let parent_venv_path = &self.venv_path.parent();
        let parent_venv_path = parent_venv_path.unwrap();
        println!(
            "Creating virtualenv in: {}",
            self.venv_path.to_string_lossy()
        );
        std::fs::create_dir_all(&parent_venv_path);
        let status = std::process::Command::new("python")
            .args(&["-m", "venv", &self.venv_path.to_string_lossy()])
            .status()
            .expect("Failed to start python process");
        if !status.success() {
            println!("Failed to create virtualenv");
        }
        // Always upgrade pip right after creation. We use 'venv' from the
        // stdlib, so the pip in the virtualenv is the one bundle with python
        // which in all likelyhood is old
        self.upgrade_pip();
    }

    fn run_pip_freeze(&self) {
        let python = self.venv_path.join("bin/python");
        let command = std::process::Command::new(python)
            .args(&["-m", "pip", "freeze", "--exclude-editable"])
            .output();
        let command = command.unwrap();
        if !command.status.success() {
            eprintln!("pip freeze failed");
        }
        let to_write = command.stdout;
        std::fs::write("requirements.lock", &to_write);
        println!("Requirements written to requirements.lock");
    }

    fn run_pip_install(&self) {
        let python = self.venv_path.join("bin/python");
        let command = std::process::Command::new(python)
            .args(&[
                "-m",
                "pip",
                "install",
                "--requirement",
                &self.requirements_lock_path.to_string_lossy(),
            ]).status();
        let command = command.unwrap();
        if !command.success() {
            eprintln!("pip install failed");
        }
    }

    pub fn upgrade_pip(&self) {
        let python = self.venv_path.join("bin/python");
        let status = std::process::Command::new(python)
            .args(&["-m", "pip", "install", "pip", "--upgrade"])
            .status()
            .expect("Failed to start python process");
        if !status.success() {
            println!("Failed upgrade pip");
        }
    }

    fn install_editable(&self) {
        let python = self.venv_path.join("bin/python");
        let status = std::process::Command::new(python)
            .args(&["-m", "pip", "install", "-e.[dev]"])
            .status()
            .expect("Failed to start python process");
        if !status.success() {
            println!("Failed to run python setup.py develop");
        }
    }
}
