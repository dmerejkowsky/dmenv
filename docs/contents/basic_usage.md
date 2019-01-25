# Usage

## Setup

First, `dmenv` needs a Python3 interpreter in PATH, which should be called `python` or `python3`. This should already be the case if you've just installed Python3, regardless of your operating system.

Second, `dmenv` needs a `setup.py` file to work.

* If you don't have a `setup.py` yet, you can run `dmenv init <project name>`
  to generate one. In this case, make sure to read the comments inside
  and edit it to fit your needs.

* If you already have one, please note that `dmenv` uses the `extras_require` keyword with a `dev` key
  to specify development dependencies, which you can use to replace your `dev-requirements.txt`
  file for instance.

In both cases, here are the contents of `setup.py` file you should end up with:

```python
from setuptools import setup

setup(
    name="demo",
    version="0.6.1",
    ...
    install_requires=[
        "path.py",
    ],
    extras_require={
        "dev": [
            "pytest",
        ],
    },
    ...
)
```


You are now ready to use `dmenv`. Keep on reading about the two main commands: `dmenv lock` and `dmenv install`.


## dmenv lock

Here's what `dmenv lock` does:

* It looks for a binary named `python3` or `python` in the `PATH` environment variable.
* It runs a bit of Python code to determine the interpreter version (3.6, 3.7 ...).
* Then, it creates a virtual environment in `.venv/<version>` using `python -m venv`.
  (This step is skipped if `dmenv` detects it is run from an existing virtual environment).
  Note that you may have to [configure other tools](./advanced_usage.md#configuring-other-tools) to ignore this directory.


* Then it runs `pip intall --editable .[dev]` so that your dev dependencies are
  installed, and the scripts listed in `entry_points` are created.

* Finally, it runs `pip freeze` to generate a `requirements.lock` file.

Now you can add the `requirements.lock` file to your version control system.


This leads us to the next command:

## dmenv install

Now that the complete list of dependencies and their versions is written in the
`requirements.lock` file, anyone can run `dmenv install` to install all the
dependencies and get exactly the same versions you got when you ran `dmenv lock`.

Hooray reproducible builds!


## Configuring other tools

Depending of your usage, you may need to tell other tools to ignore the `.venv` directory.

* **git**: add a line containing `.venv/` to the `.gitgnore`.

```text
# should be already there if you use
# setup.py
*.egg-info
build/
dist/

# only directory in which `dmenv` will write files:
.venv/
```

* **pyflakes**, **pylint** and other linters: add some configuration in the `setup.cfg` file:

```ini
exclude =
  .venv
```

As an alternative, you can also ask `dmenv` to create its virtual environment *outside* your project,
by setting the `DMENV_VENV_OUTSIDE_PROJECT` environment variable. It will then use
the [app_dirs crate](https://crates.io/crates/app_dirs) as a location to store the created virtual environments.

## Going further

That's all for the basic usage of `dmenv`, you may proceed to the [goodies section](./goodies.md) or read on about [advanced dmenv usage](./advanced_usage.md)
