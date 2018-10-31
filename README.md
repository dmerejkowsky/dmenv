# dmenv: the stupid virtualenv manager



## Setup


`dmenv` only needs one config file named `dmenv.toml`, located in `~/.config`
on Linux and macOS, and `~/AppData/Local` on Windows.

The config **must** contain a `default` environment, like this:

```
[env.default]
python = /path/to/python
```

Then, make sure to have `setup.py` file looking like this:

```python
setup(
  name="foo",
  version="0.1",
  install_requires=[
    # Your deps here
    "bar",
    "baz >= 2.0",
  ],
  extras_require = {
    # Your dev deps here
    "dev": [
      "pytest",
    ]
  },
  entry_points={
     "console_scripts": [
        "foo = foo:main"
      ]
  },
)
```

Now you are ready to use `dmenv`!

## freeze

Here's what `dmenv freeze` does:

* Create a virtualenv for you with `python -m venv` in `.venv/default`. (Make sure to add `.venv` to your `.gitignore`).
* Run `pip intall --editable .[dev]` so that your dev deps are installed, and the scripts listed in `entry_points` are
  created.
* Run `pip freeze` to generate a `requirements.lock` file.

## install

Now you can add `requirements.lock` to your git repo, and then anyone can run `dmenv install` to install all the deps and get exactly the same versions you got when you ran `dmenv freeze`. Hooray reproducible builds!

## run

As a convenience, you can use:`dmenv run` to run any binary from the virtualenv

## show

`dmenv show` will show you the path of the virtualenv. No more, no less.

On Linux, you might use something like `source $(dmenv show)/bin/activate` to activate the virtualenv in your shell.

## upgrade-pip

Tired of `pip` telling you to upgrade itself? Run `dmenv upgrade-pip` :)

It's exactly the same as typing `dmenv run -- python -m pip install --upgrade pip`, but with less keystrokes :P

## Using an other python version

To use a different Python version, add a new section in the `dmenv.toml` config file with the name and the path to the binary, like this:

```toml
[env.3.8]
python = "/path/to/python3.8"
```
Then you can use Python 3.8 with all the `dmenv` commands by prefixing them with `dmenv --env 3.8`.

An other virtualenv will be used in `.venv/3.8` so that you can keep your default virtualenv in `.venv/default`.

Cool, no?


# FAQ

Q: How do I add dependencies to build the documentation?<br/>
A: Stick them in the `dev` section.

Q: What if I don't want to install the dev dependencies?<br/>
A: Don't use dmenv. Run `pip install` without `[dev]` extras. If you insist, maybe a `--no-dev` option will be added.

Q: How do I upgrade a dependency?<br/>
A: Just run `dmenv freeze` again. If something breaks, either fix your code or use more precise version specifiers

Q: How do I depend on a git specific repo/branch?<br/>
A: Edit the `requirements.lock` by hand like this:

```
foo==0.1
https://gitlab.com/foo/bar@my-branch
```

Q: But that sucks and it will disappear when I re-run `dmenv freeze`! <br />
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
