use colored::*;
use regex::Regex;
use structopt::StructOpt;

use crate::error::Error;

#[derive(StructOpt)]
#[structopt(
    name = "dmenv",
    about = "Simple and practical virtualenv manager for Python"
)]
pub struct Command {
    #[structopt(long = "python", help = "python binary")]
    pub python_binary: Option<String>,

    #[structopt(long = "project", help = "path to use as the project directory")]
    pub project_path: Option<String>,

    #[structopt(subcommand)]
    pub sub_cmd: SubCommand,
}

fn parse_python_version(string: &str) -> Result<String, Error> {
    let re = Regex::new("^(==|<|<=|>|>=) (('.*?')|(\".*?\"))$").unwrap();
    if !re.is_match(string) {
        return Err(Error::Other {
            message: "should match something like `<= '3.6'`".to_string(),
        });
    }
    Ok(string.to_string())
}

#[derive(StructOpt)]
pub enum SubCommand {
    #[structopt(name = "clean", about = "clean existing virtualenv")]
    Clean {},

    #[structopt(name = "develop", about = "run setup.py develop")]
    Develop {},

    #[structopt(name = "install", about = "Install all dependencies")]
    Install {
        #[structopt(long = "--no-develop", help = "do not run setup.py develop")]
        no_develop: bool,
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

        #[structopt(long = "author", help = "author")]
        author: Option<String>,
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

    #[structopt(name = "show:deps", about = "Show dependencies information")]
    ShowDeps {},

    #[structopt(name = "show:venv_path", about = "Show path of the virtualenv")]
    ShowVenvPath {},

    #[structopt(
        name = "show:bin_path",
        about = "Show path of the virtualenv's binaries"
    )]
    ShowVenvBin {},

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_python_version_ok() {
        assert_eq!("< '3.6'", parse_python_version("< '3.6'").unwrap());
    }

    #[test]
    fn test_parse_python_version_no_comparison() {
        parse_python_version("3.6").unwrap_err();
    }

    #[test]
    fn test_parse_python_version_not_quoted() {
        parse_python_version("<= 3.6").unwrap_err();
    }
}
