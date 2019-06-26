use crate::cmd::*;
use crate::error::*;
use std::path::PathBuf;

pub struct InitOptions {
    name: String,
    version: String,
    author: Option<String>,
    setup_cfg: bool,
}

impl InitOptions {
    pub fn new(name: &str, version: &str, author: &Option<String>) -> Self {
        #![allow(clippy::redundant_closure)]
        InitOptions {
            name: name.to_string(),
            version: version.to_string(),
            author: author.as_ref().map(|x| x.to_string()),
            setup_cfg: true,
        }
    }

    pub fn no_setup_cfg(&mut self) {
        self.setup_cfg = false
    }
}

fn ensure_path_does_not_exist(path: &PathBuf) -> Result<(), Error> {
    if path.exists() {
        return Err(Error::FileExists {
            path: path.to_path_buf(),
        });
    }
    Ok(())
}

fn write_to_path(path: &PathBuf, contents: &str) -> Result<(), Error> {
    std::fs::write(path, contents).map_err(|e| new_write_error(e, path))
}

pub fn init(project_path: &PathBuf, options: &InitOptions) -> Result<(), Error> {
    let setup_cfg_path = project_path.join("setup.cfg");
    let setup_py_path = project_path.join("setup.py");

    // A setup.py is written in both cases, so check we're not
    // overwriting it first
    ensure_path_does_not_exist(&setup_py_path)?;

    if options.setup_cfg {
        ensure_path_does_not_exist(&setup_cfg_path)?;
        write_from_template(include_str!("setup.in.cfg"), &setup_cfg_path, options)?;
    } else {
        write_from_template(include_str!("setup.in.py"), &setup_py_path, options)?;
    }

    if options.setup_cfg {
        // We still need an almost-empty setup.py even when all the configuration
        // is in setup.cfg :/
        write_to_path(&setup_py_path, "from setuptools import setup\nsetup()\n")?;
        print_info_1("Project initialized with setup.py and setup.cfg files");
    } else {
        print_info_1("Project initialized with a setup.py file");
    }
    Ok(())
}

fn write_from_template(
    template: &str,
    dest_path: &PathBuf,
    options: &InitOptions,
) -> Result<(), Error> {
    // Warning: make sure the template files in `src/operations/` contain all those
    // placeholders
    let with_name = template.replace("<NAME>", &options.name);
    let with_version = with_name.replace("<VERSION>", &options.version);
    let to_write = if let Some(ref author) = options.author {
        with_version.replace("<AUTHOR>", author)
    } else {
        with_version
    };
    write_to_path(dest_path, &to_write)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempdir;

    #[test]
    fn creates_two_files_by_default() {
        let tmp_dir = tempdir::TempDir::new("test-dmenv-init").unwrap();
        let tmp_path = tmp_dir.path();

        run_init(&tmp_path.to_path_buf()).unwrap();

        let setup_py = std::fs::read_to_string(&tmp_path.join("setup.py")).unwrap();
        assert_contains(&setup_py, "setup()");
        assert_not_contains(&setup_py, "foo");

        let setup_cfg = std::fs::read_to_string(&tmp_path.join("setup.cfg")).unwrap();
        assert_contains(&setup_cfg, "name = foo");
        assert_contains(&setup_cfg, "version = 0.42");
    }

    #[test]
    fn no_setup_cfg() {
        let temp_dir = tempdir::TempDir::new("test-dmenv-init").unwrap();
        let tmp_path = temp_dir.path();

        run_init_no_setup_cfg(&tmp_path.to_path_buf()).unwrap();

        let setup_cfg_path = tmp_path.join("setup.cfg");
        assert!(!setup_cfg_path.exists());

        let setup_py = std::fs::read_to_string(tmp_path.join("setup.py")).unwrap();
        assert_contains(&setup_py, "\"foo\"");
        assert_contains(&setup_py, "\"0.42\"");
    }

    #[test]
    fn does_not_overwrite_setup_cfg() {
        let temp_dir = tempdir::TempDir::new("test-dmenv-init").unwrap();
        let tmp_path = temp_dir.path();
        let setup_cfg_path = tmp_path.join("setup.cfg");
        touch(&setup_cfg_path);

        let err = run_init(&tmp_path.to_path_buf()).unwrap_err();
        assert_file_exists_error(err, &setup_cfg_path);
    }

    #[test]
    fn does_not_overwrite_setup_py() {
        let temp_dir = tempdir::TempDir::new("test-dmenv-init").unwrap();
        let tmp_path = temp_dir.path();
        let setup_py_path = tmp_path.join("setup.py");
        touch(&setup_py_path);

        let err = run_init_no_setup_cfg(&tmp_path.to_path_buf()).unwrap_err();
        assert_file_exists_error(err, &setup_py_path);
    }

    fn assert_contains(text: &str, sub_string: &str) {
        if !text.contains(sub_string) {
            panic!("\n{}should contain {}", text, sub_string);
        }
    }

    fn assert_not_contains(text: &str, sub_string: &str) {
        if text.contains(sub_string) {
            panic!("\n{}should not contain {}", text, sub_string);
        }
    }

    fn assert_file_exists_error(err: Error, expected_path: &PathBuf) {
        match err {
            Error::FileExists { path } => assert_eq!(&path, expected_path),
            _ => panic!("Expecting FileExists, got: {}", err),
        }
    }

    fn touch(path: &PathBuf) {
        std::fs::write(&path, "# don't overwrite me").unwrap()
    }

    fn run_init(tmp_path: &PathBuf) -> Result<(), Error> {
        let init_options = InitOptions::new("foo", "0.42", &None);
        init(tmp_path, &init_options)
    }

    fn run_init_no_setup_cfg(tmp_path: &PathBuf) -> Result<(), Error> {
        let mut init_options = InitOptions::new("foo", "0.42", &None);
        init_options.no_setup_cfg();
        init(tmp_path, &init_options)
    }

}
