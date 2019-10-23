use colored::*;
use regex::Regex;
use structopt::StructOpt;

use crate::error::*;

#[derive(StructOpt, Debug)]
#[structopt(
    name = "dmenv",
    about = "Simple and practical virtualenv manager for Python"
)]
pub struct Command {
    #[structopt(long = "python", help = "python binary")]
    pub python_binary: Option<String>,

    #[structopt(long = "project", help = "path to use as the project directory")]
    pub project_path: Option<String>,

    #[structopt(long = "production", help = "Ignore dev dependencies")]
    pub production: bool,

    #[structopt(subcommand)]
    pub sub_cmd: SubCommand,
}

#[derive(StructOpt, Debug)]
pub enum SubCommand {
    #[structopt(name = "clean", about = "Clean existing virtualenv")]
    Clean {},

    #[structopt(name = "develop", about = "Run setup.py develop")]
    Develop {},

    #[structopt(name = "install", about = "Install all dependencies")]
    Install {
        #[structopt(long = "--no-develop", help = "Do not run setup.py develop")]
        no_develop: bool,
        #[structopt(
            long = "--system-site-packages",
            help = "Give the virtual environment access to the system site-packages dir"
        )]
        system_site_packages: bool,
    },

    #[structopt(name = "bump-in-lock", about = "Bump a dependency in the lock file")]
    BumpInLock {
        #[structopt(help = "name")]
        name: String,

        #[structopt(long = "--git")]
        git: bool,

        #[structopt(help = "version")]
        version: String,
    },

    #[structopt(name = "init", about = "Initialize a new project")]
    Init {
        #[structopt(help = "Project name")]
        name: String,

        #[structopt(long = "version", help = "Project version", default_value = "0.1.0")]
        version: String,

        #[structopt(long = "author", help = "Author name")]
        author: Option<String>,

        #[structopt(
            long = "no-setup-cfg",
            help = "Keep all code in the `setup.py` file, do not use `setup.cfg`"
        )]
        no_setup_cfg: bool,
    },

    #[structopt(name = "lock", about = "(Re)-generate requirements.lock")]
    Lock {
        #[structopt(
            long = "python-version",
            help = "Restrict Python version",
            parse(try_from_str = "parse_python_version")
        )]
        python_version: Option<String>,

        #[structopt(long = "platform", help = "Restrict platform")]
        sys_platform: Option<String>,

        #[structopt(
            long = "--system-site-packages",
            help = "Give the virtual environment access to the system site-packages dir"
        )]
        system_site_packages: bool,
    },

    #[structopt(name = "run", about = "Run the given binary from the virtualenv")]
    Run {
        #[structopt(
            long = "--no-exec",
            help = "On Unix, fork a new process instead of using exec(). On Windows, this is a no op"
        )]
        no_exec: bool,

        #[structopt(name = "command")]
        cmd: Vec<String>,
    },

    #[structopt(name = "process-scripts", help = "Process generated scripts")]
    ProcessScripts {
        #[structopt(long = "--force", help = "force override of existing files")]
        force: bool,
    },

    #[structopt(name = "show:deps", about = "Show installed dependencies information")]
    ShowDeps {},

    #[structopt(
        name = "show:outdated",
        about = "Show outdated dependencies information"
    )]
    ShowOutDated {},

    #[structopt(name = "show:venv_path", about = "Show path of the virtualenv")]
    ShowVenvPath {},

    #[structopt(
        name = "show:bin_path",
        about = "Show path of the virtualenv's binaries"
    )]
    ShowVenvBin {},

    #[structopt(name = "tidy", about = "Re-generate a clean lock")]
    Tidy {},

    #[structopt(name = "upgrade-pip", about = "Upgrade pip in the virtualenv")]
    UpgradePip {},
}

pub fn print_error(description: &str) {
    eprintln!("{}: {}", "Error".bold().red(), description);
}

pub fn print_warning(description: &str) {
    eprintln!("{}: {}", "Warning".bold().yellow(), description);
}

pub fn print_info_1(message: &str) {
    println!("{} {}", "::".blue(), message);
}

pub fn print_info_2(message: &str) {
    println!("{} {}", "->".blue(), message);
}

// Make sure the `--python-version` option used in `dmenv lock`
// can be written as marker in the lock file
fn parse_python_version(string: &str) -> Result<String, Error> {
    // Note: parsing *all* the possible syntaxes is a hard problem
    // (see https://www.python.org/dev/peps/pep-0508/#grammar for details),
    // so we use a regex that matches a *subset* of what is possible
    // instead.
    let re = Regex::new("^(==|<|<=|>|>=) (('.*?')|(\".*?\"))$").unwrap();
    if !re.is_match(string) {
        return Err(new_error(
            "should match something like `<= '3.6'`".to_string(),
        ));
    }
    Ok(string.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use spectral::prelude::*;

    #[test]
    fn test_parse_python_version_ok() {
        assert_that!(parse_python_version("< '3.6'"))
            .is_ok()
            .is_equal_to("< '3.6'".to_string());
    }

    #[test]
    fn test_parse_python_version_no_comparison() {
        assert_that!(parse_python_version("3.6")).is_err();
    }

    #[test]
    fn test_parse_python_version_not_quoted() {
        assert_that!(parse_python_version("<= 3.6")).is_err();
    }
}
