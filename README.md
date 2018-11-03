# dmenv: the stupid virtualenv manager

<a href="https://crates.io/crates/dmenv"><img src="https://img.shields.io/crates/v/dmenv.svg"/></a>
[![Build](https://img.shields.io/travis/dmerejkowsky/dmenv.svg?branch=master)](https://travis-ci.org/dmerejkowsky/dmenv)

## Installation

Download the [dmenv installer](https://raw.githubusercontent.com/dmerejkowsky/dmenv/master/installer.py), then run
`python installer.py`, or `python3 installer.py`, depending on how your Python interpreter is called. Just make
sure it's Python3, not 2.

The script will fetch pre-compiled binaries from GitHub. If you prefer, you can also [install rust](https://www.rust-lang.org/en-US/install.html) and install dmenv with `cargo install dmenv`.

## Setup

In order to run, `dmenv` needs to know the path to the Python3 interpreter you will be using.

To do so, run:

```console
$ dmenv pythons add default /path/to/python3
```

Then, `dmenv` needs a `setup.py` file. If you don't have one yet, run

`dmenv init --name <project name>` to generate one. Make sure to read the comments inside and edit it to fit your needs.

Now you are ready to use `dmenv`!

Here's a description of the main commands:

## dmenv lock

Here's what `dmenv lock` does:

* Create a virtualenv for you with `python -m venv` in `.venv/default`. (Make sure to add `.venv` to your `.gitignore`).
* Run `pip intall --editable .[dev]` so that your dev deps are installed, and the scripts listed in `entry_points` are
  created.
* Run `pip freeze` to generate a `requirements.lock` file.

Now you can add the `requirements.lock` file to your version control system.

This leads us to the next command.

## dmenv install

Now that the complete list of dependencies and their versions is written in the
`requirements.lock` file, anyone can run `dmenv install` to install all the
dependencies and get exactly the same versions you got when you ran `dmenv lock`.

Hooray reproducible builds!

## dmenv run

As a convenience, you can use:`dmenv run` to run any binary from the virtualenv. If the program you want to run
needs command-line options, use a `--` to separated them from `dmenv` options, like so:

```console
dmenv run -- pytest --collect-only
```

## dmenv upgrade-pip

Tired of `pip` telling you to upgrade itself? Run `dmenv upgrade-pip` :)

It's exactly the same as typing `dmenv run -- python -m pip install --upgrade pip`, but with less keystrokes :P

## Using an other python version

To use a different Python version, run `dmenv pythons add <version> <path>`, when `path` is the full
path to the python binary. For instance: `dmenv pythons add 3.8 /path/to/python3.8`.

Then you can use Python 3.8 with all the `dmenv` commands by prefixing them with `dmenv --env 3.8`.

An other virtualenv will be used in `.venv/3.8` so that you can keep your default virtualenv in `.venv/default`.

Cool, no?

# FAQ

Q: How do I upgrade a dependency?<br/>
A: Just run `dmenv lock` again. If something breaks, either fix your code or use more precise version specifiers in `setup.py`, like `foobar < 2.0`.

Q: How do I depend on a git specific repo/branch?<br/>
A: Edit the `requirements.lock` by hand like this:

```
foo==0.1
https://gitlab.com/foo/bar@my-branch
```

Q: But that sucks and it will disappear when I re-run `dmenv lock`! <br />
A: See #7. We are looking for a proper solution. In the mean time, feel free to:

  * Open a pull request if you've forked an upstream project
  * Use a local pipy mirror and a little bit of CI to publish your sources there


Q: Why Rust? <br />
A:

* Because it has excellent support for what we need: manipulate paths and run commands in a cross-platform way
* Because it's my second favorite language
* Because distribution is really easy
* Because by *not* using Python I'm less likely to depend on pip or virtualenv's internals
