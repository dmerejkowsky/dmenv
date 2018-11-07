# dmenv: the stupid virtualenv manager

<a href="https://crates.io/crates/dmenv"><img src="https://img.shields.io/crates/v/dmenv.svg"/></a>
[![Build](https://img.shields.io/travis/TankerHQ/dmenv.svg?branch=master)](https://travis-ci.org/TankerHQ/dmenv)

## Installation

The easiest way is to download the matching binary from the [releases page](https://github.com/TankerHQ/dmenv/releases) for your platform and put it
somewhere on in your $PATH.

If you prefer, you can also [install rust](https://www.rust-lang.org/en-US/install.html) and install dmenv with `cargo install dmenv`.

## Setup

First, `dmenv` needs a Python3 interpreter in PATH, which should be called `python` or `python3`. This should already be the case if you've just installed Python3, regardless of your operating system.

Second, `dmenv` needs a `setup.py` file to work.

* If you don't have a `setup.py` yet, you can run `dmenv init <project name>`
  to generate one. In this case, make sure to read the comments inside
  and edit it to fit your needs.

* If you already have one, please note that `dmenv` uses the `extras_require` keyword with a `dev` key
  to specify development dependencies, which you can use to replace your `dev-requirements.txt`
  file for instance.

And that's it. Now you are ready to use `dmenv`!

Here's a description of the main commands:

## dmenv lock

Here's what `dmenv lock` does:

* First, it creates a virtualenv for you with `python -m venv` in
  `.venv/<version>`, where `<version>` is read from `python --version`. Make
  sure to add `.venv` to your `.gitignore`! Note that this step is skipped
  if `dmenv` detects it is run from an existing virtualenv.

* Then it runs `pip intall --editable .[dev]` so that your dev deps are installed, and the scripts listed in `entry_points` are
  created.

* Finally, it runs `pip freeze` to generate a `requirements.lock` file.

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

## Using an other python interpreter

To use an other Python interpreter than the one in PATH, you can either:

* Modify your PATH environment variable so that it appears there. (For instance, with [pyenv](https://github.com/pyenv/pyenv)).
* Prefix all the `dmenv` commands with a `--python /path/to/other/python` flag.

# FAQ

Q: How do I upgrade a dependency?<br/>
A: Just run `dmenv lock` again. If something breaks, either fix your code or
   use more precise version specifiers in `setup.py`, like `foobar < 2.0`.

Q: How do I depend on a git specific repo/branch?<br/>
A: Edit the `requirements.lock` by hand like this:

```
foo==0.1
https://gitlab.com/foo/bar@my-branch
```

Q: But that sucks and it will disappear when I re-run `dmenv lock`! <br />
A: See [#7](https://github.com/TankerHQ/dmenv/issues/7). We are looking for a proper solution. In the mean time, feel free to:

  * Open a pull request if you've forked an upstream project
  * Use a local pipy mirror and a little bit of CI to publish your sources there


Q: Why Rust? <br />
A:

* Because it has excellent support for what we need: manipulate paths and run commands in a cross-platform way
* Because it's my second favorite language
* Because distribution is really easy
* Because by *not* using Python at all `dmenv` is less likely to break if something on your system changes.
