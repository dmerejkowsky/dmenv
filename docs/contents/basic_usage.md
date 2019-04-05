# Usage

## Setup

First, `dmenv` needs a Python3 interpreter in PATH, which should be called `python` or `python3`. This should already be the case if you've just installed Python3, regardless of your operating system.

Second, `dmenv` needs a `setup.py` file to work.

* If you don't have a `setup.py` yet, you can run `dmenv init <project name>`
  to generate one, alongside a `setup.cfg` file. In this case, make sure to read the comments inside
  and edit it to fit your needs.

* If you already have a `setup.py` or a `setup.cfg` file that contains info about dependencies, please note that `dmenv` the
 **extras require** dependencies to specify development dependencies.

You are now ready to use `dmenv`. Keep on reading about the two main commands: `dmenv lock` and `dmenv install`.


## dmenv lock

Here's what `dmenv lock` does:

* It looks for a binary named `python3` or `python` in the `PATH` environment variable.
* It runs a bit of Python code to determine the interpreter version (3.6, 3.7 ...).
* Then, it creates a virtual environment in `.venv/dev/<version>` using `python -m venv`.
  (This step is skipped if `dmenv` detects it is run from an existing virtual environment).
  Note that you may have to [configure other tools](./advanced_usage.md#configuring-other-tools) to ignore this directory.


* Then it runs `pip install --editable .[dev]` so that your dev dependencies are
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

* **git**: add a line containing `.venv/` to the `.gitignore`.

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
by setting the `DMENV_VENV_OUTSIDE_PROJECT` environment variable to a non-empty value like `1`. It will then use
the [app_dirs crate](https://crates.io/crates/app_dirs) as a location to store the created virtual environments.

## Upgrading dmenv

If you have `wget` installed and used a pre-compiled binary, upgrading `dmenv` can be done in just one command:

```bash
# Replace <version> and <arch> with their correct value
$ wget \
  https://github.com/TankerHQ/dmenv/releases/download/<version>/dmenv-<arch> \
  -O $(which dmenv)
```

If you've installed `dmenv` from source:

```bash
rustup update stable  # dmenv usually requires latest rust stable version
cargo install --force dmenv
```


## Upgrading a top-level dependency

Let's say your project depends on `foolib`. Version `1.3` works fine, but
`1.4` just came out and even if you don't need any new feature from `1.4`,
you'd like to check wether your project is compatible. [^1]

Assuming you already have a virtual environment containing `foolib 1.3`, you can can do so by running:

```bash
$ git status  # check that requirements.lock is clean
$ dmenv run -- pip install --upgrade foolib
```

Most of the time, `pip install --upgrade foolib` will do the right thing:

* it will keep other dependencies installed at their current version
* it will install new dependencies of `foolib 1.4` if they are missing
* if `foolib 1.4` contains a `bar >= 3.0` constraint and you have `bar == 2.0` in the virtualenv, `bar` will
  be upgraded too.

Then it's time to register the new dependencies in the lock:

```bash
$ dmenv lock
```

You can now inspect the differences in the lock file by hand, and if they are correct, commit and push a new version of the lock file.

## Going further

That's all for the basic usage of `dmenv`, you may proceed to the [goodies section](./goodies.md) or read on about [advanced dmenv usage](./advanced_usage.md)


[^1]: If you *do* need `foolib` in version 1.4 or later, you should express this constraint in the setup.py file instead, as explained in the *[upgrading juste one regular dependency](./advanced_usage.md#upgrading_just_one_development_dependency)*
section.
