# dmenv: the stupid virtualenv manager

## Installation

Download the [dmenv installer](https://raw.githubusercontent.com/dmerejkowsky/dmenv/master/installer.py), then run
`python installer.py`, or `python3 installer.py`, depending on how your Python interpreter is called. Just make
sure it's Python3, not 2.

The script will fetch pre-compiled binaries from GitHub. If you don't like that, use `cargo install dmenv` to build
from the sources.

## Setup

In order to run, `dmenv` needs to know the path to the Python3 interpreter you will be using.

To do so, run:

```console
$ dmenv add default /path/to/python3
```

Then, `dmenv` needs a `setup.py` file. If you don't have one yet, run

`dmenv init --name <project name>` to generate one. Make sure to read the comments inside and edit it to fit your needs.

Now you are ready to use `dmenv`!

## lock

Here's what `dmenv lock` does:

* Create a virtualenv for you with `python -m venv` in `.venv/default`. (Make sure to add `.venv` to your `.gitignore`).
* Run `pip intall --editable .[dev]` so that your dev deps are installed, and the scripts listed in `entry_points` are
  created.
* Run `pip freeze` to generate a `requirements.lock` file.

Now you can add `requirements.lock`, to your version control system, which leads us to the next command.

## install

Now that the complete list of dependencies and their versions is written in the
`requirements.lock` file, anyone can run `dmenv install` to install all the
dependencies and get exactly the same versions you got when you ran `dmenv lock`.

Hooray reproducible builds!

## run

As a convenience, you can use:`dmenv run` to run any binary from the virtualenv

## show

`dmenv show` will show you the path of the virtualenv. No more, no less.

On Linux, you might use something like `source $(dmenv show)/bin/activate` to activate the virtualenv in your shell.

## upgrade-pip

Tired of `pip` telling you to upgrade itself? Run `dmenv upgrade-pip` :)

It's exactly the same as typing `dmenv run -- python -m pip install --upgrade pip`, but with less keystrokes :P

## Using an other python version

To use a different Python version, run `dmenv env pythons add <version> <path>`, when `path` is the full
path to the python binary.

(Alternatively, you can also edit the `dmenv.toml` config file, which should be located in `~/.config` and Linux and
macOS, or  `%HOME%/AppData/Local` on Windows.
Then you can use Python 3.8 with all the `dmenv` commands by prefixing them with `dmenv --env 3.8`.

An other virtualenv will be used in `.venv/3.8` so that you can keep your default virtualenv in `.venv/default`.

Cool, no?

# Troubleshooting

You may get this error message when using `dmenv` with old Python:
```
-> running /mnt/data/dmerej/src/dmenv/demo/.venv/3.6/bin/python -m pip freeze --exclude-editable
Error pip freeze failed:
Usage:
  /mnt/data/dmerej/src/dmenv/demo/.venv/3.6/bin/python -m pip freeze [options]

no such option: --exclude-editable
```

The clue to the error is located right above:

```
You are using pip version 9.0.3, however version 18.1 is available.
```

Run `dmenv upgrade-pip` and the problem should go away.

(And learn to read **the whole output** of the commands you run :)



# FAQ

Q: How do I add dependencies to build the documentation?<br/>
A: Stick them in the `dev` section.

Q: What if I don't want to install the dev dependencies?<br/>
A: Don't use dmenv. Run `pip install` without `[dev]` extras. If you insist, maybe a `--no-dev` option will be added.

Q: How do I upgrade a dependency?<br/>
A: Just run `dmenv lock` again. If something breaks, either fix your code or use more precise version specifiers

Q: How do I depend on a git specific repo/branch?<br/>
A: Edit the `requirements.lock` by hand like this:

```
foo==0.1
https://gitlab.com/foo/bar@my-branch
```

Q: But that sucks and it will disappear when I re-run `dmenv lock`! <br />
A: Yes that sucks. Feel free to:
  * Open a pull request if you've forked an upstream project
  * Use a local pipy mirror and a little bit of CI to publish your sources there



## Why?

* Because pipenv, poetry and tox are too big and too complex
* Because virtualenv + requirements.txt has worked for 10 years and will continue to work for 10 years
* Because it will continue to work if / when pip supports pipfile
* Because dependency management is very hard, and pip already does a good enough job

## Why Python3 only?

* Because it's 2018

## Why not use virtualenv?

* Because python3 -m venv works since Python3.3, except on debian where you have to run `apt install python3-venv`. But that's Debian's problem, not mine

## But I don't want to maintain a `setup.py`!

Too bad. Don't use dmenv, then. poetry is cool.

## Why Rust?

* Because it has excellent support for what we need: manipulate paths and run commands in a cross-platform way
* Because it's my second favorite language
* Because distribution is really easy
* Because by *not* using Python I'm less likely to depend on pip or virtualenv's internals
