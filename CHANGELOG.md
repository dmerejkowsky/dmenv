# Next release

* Do not change the working directory when creating a virtual environment. This makes combining `dmenv` and `pyenv` much easier.
* Add `dmenv create` command to create an empty virtual environment.

# 0.19.0 (2019-O6-11)

* **Breaking**: the `--system-site-packages` option must be passed *before* any subcommand, and the path of the virtual environment has changed:

**<= 0.18**:
```
$ dmenv install --system-site-packages
# virtual environment created in .venv/dev/3.7/
```

**>= 0.19**:
```
$ dmenv --system-site-packages  install
# virtual environment created in .venv/dev-system/3.7/
```

* Fix #110: `dmenv tidy` now works even from an activated virtual environment.

# 0.18.0 (2019-29-10)

* Add a `tidy` command, to re-generate locks from a clean virtual environment.

# 0.17.0 (2019-10-08)

* Add a `process-scripts` command, to generate scripts in `DMENV_SCRIPTS_PATH`.

# 0.16.2 (2019-10-03)

* Fix regression: `dmenv init` could only be used with the `--project` option
* Improve error messages when setup.py or requirements.lock is not found
* Improve error handling when parsing output from `info.py`
* Fix message when using `dmenv --production install`

# 0.16.1 (2019-07-24)

* Fix regression that caused parsing output of `dmenv show` commands to stop working

# 0.16.0 (2019-07-24)

* Fix #94: Look for `setup.py` in the parent directories when trying to resolve the project path.

# 0.15.0 (2019-06-26)

* When `foo.py` is present at the root of the project, you can us `dmenv run foo.py` directly instead of
  the more awkward `dmenv run -- python foo.py`

# 0.14.3 (2019-05-15)

* Fix syntax of generated setup.py (#86)
* Fix regression: keep the top comment when running bump-in-lock

# 0.14.2 (2019-04-29)

* Restore generation of pre-compiled binaries from travis.

# 0.14.1 (2019-04-29)

* Fix incorrect `--help` message
* Better error handling when the virtual environment or binary path do not exist
* Fix typo in `setup.cfg` template

All reported by @theodelrieu. Thanks, man.

# 0.14.0 (2019-04-05)

## Breaking: `dmenv init` now uses a separate setup.cfg file alongside setup.py

This leads to far more readable code.

Note: this may break when using setuptools <= 30.3.0 (roughly Python 3.5). Use `dmenv init --no-setup-cfg` if you
need compatibility with old Python versions.

# 0.13.0 (2019-04-03)

* Implement [#77](https://github.com/TankerHQ/dmenv/issues/77): Add a new `--production` flag to use `prod` extra requirements instead of `dev`. This allows having dependencies _just_ for production environments.

# 0.12.0 (2018-03-20)

## Show outdated dependencies

* Use `dmenv show:outdated`  to show outdated dependencies.

## Allow access to system site packages

* `dmenv install` and `dmenv lock` commands learned the `--system-site-packages` option to create virtual environments that have access to packages installed globally on the system.

## Allow skipping development dependencies

This is done with the `--production` flag. For instance, `dmenv --production install`.
`dmenv --production lock` will create a `production.lock` that contains no development dependencies.

## Breaking changes

Virtual environment location has changed to allow both production and full virtual environments to coexist:

* When using `DMENV_VENV_OUTSIDE_PROJECT`

| version | location |
|-|----------|
| <= 0.11 | DATA_DIR/dmenv/venv/3.7.1/foo/
| >= 0.12, default | DATA_DIR/dmenv/venv/dev/3.7.1/foo/
| >= 0.12, with --production |  DATA_DIR/dmenv/venv/prod/3.7.1/foo/


* Otherwise:

| version | location |
|-|----------|
| <= 0.11 | .venv/3.7.1/foo/ |
| >= 0.12, default | .venv/dev/3.7.1/foo/ |
| >= 0.12, with --production | .venv/prod/3.7.1/foo/ |

## Migrating from 0.11

* Run `dmenv clean` with `dmenv 0.11` to clean up the deprecated location
* Upgrade to `dmenv 0.12`
* Run `dmenv install`  to create the new virtual environment

# 0.11.1 (2010-03-01)

* Fix metadata on Cargo to include new tagline.

# 0.11.0 (2019-02-20)

* Add `dmenv show:bin_path` to show the path of the virtual environment binaries.

## Breaking changes

* Fix [#31](https://github.com/TankerHQ/dmenv/issues/31): make sure the wheel
  package gets frozen when running `dmenv lock`. Note: this also causes other packages
  like `setuptools` and `pip` itself to get frozen. As a consequence `dmenv
  install` no longer upgrades pip automatically, and so the `--no-upgrade-pip` option
  is gone.

# 0.10.0 (2019-01-30)

* Allow using `dmenv` outside the current project, by setting an environment variable named `DMENV_VENV_OUTSIDE_PROJECT`.

# 0.9.0 (2019-01-25)

* Fix [#54](https://github.com/TankerHQ/dmenv/issues/54): rename `--cwd` option to `--project`.

* Avoid blindly overwriting the `requirements.lock` file when running.
  `dmenv lock`. See [#11](https://github.com/TankerHQ/dmenv/issues/11) and [#7](https://github.com/TankerHQ/dmenv/issues/7) for background.

# 0.8.4 (2019-01-15)

* Fix [#49](https://github.com/TankerHQ/dmenv/issues/49): return code was always 0 when using `dmenv run` on Windows. (regression introduced in `0.8.1`).

# 0.8.3 (2019-01-11)

* Add documentation link to `Cargo.toml`.

# 0.8.2 (2019-01-09)

* Fix [#45](https://github.com/TankerHQ/dmenv/issues/45): `dmenv env` can be used with non-ASCII chars on Windows.

# 0.8.1 (2019-01-08)

* `dmenv run` now uses `execv` from `libc`. This means the child process is killed when killing `dmenv`.
   The previous behavior (starting a new process) can be activated with the `--no-exec` option.

# 0.8.0 (2018-12-21)

* Allow using `python3 -m virtualenv` instead of `python3 -m venv` to create the virtual
  environments by setting an environment variable named `DMENV_NO_VENV_STDLIB`. This can be used to work around
  some bugs in Debian-based distributions.

# 0.7.0 (2018-12-07)

* Add `bump-in-lock` command. Use to bump version or git references in the `requirements.lock`
  file.

# 0.6.0 (2018-11-09)

* Run `setup.py develop` with `--no-deps`.
* Rename `show` to `show:venv_path`, add `show:deps` to display the list of dependencies.

# 0.5.0 (2018-11-07)

* `dmenv init`: since name is required, it is now an argument, no longer an option.
  So instead of `dmenv init --name foo --version 0.42`, use `dmenv init foo --version 0.42`
* Add a command named `dmenv develop` that just runs `python setup.py develop` and nothing else.
* `dmenv install`: add `--no-upgrade-pip` and `--no-develop` options.

# 0.4.3 (2018-11-06)

* Add a `--author` option to `dmenv init`, used when generating the `setup.py` file.
* Fix [#12](https://github.com/TankerHQ/dmenv/issues/12): `dmenv lock` now exits immediately if the lock file is missing.
* Workaround Debian bug in pip (See [#15](https://github.com/TankerHQ/dmenv/issues/15) for details).

# 0.4.2 (2018-11-05)

* Write some metadata inside the `requirements.lock` file.

* Improve `dmenv run`:
  * Suggest running `lock` or `install`
  * Do not crash if used without arguments


# 0.4.1 (2018-11-04)

* Fix CI on Windows.

# 0.4.0 (2018-11-03)

* `dmenv` no longer needs a configuration file.
* Find the Python interpreter to use by looking in the PATH environment variable.

# 0.3.4 (2018-11-03)

* Fix [#9](https://github.com/TankerHQ/dmenv/issues/9): If `dmenv` is run *inside an existing virtual environment*, just use it.

# 0.3.3 (2018-11-03)

* Also upgrade pip when running `dmenv install`.
* Fix incorrect message when running `dmenv lock`.

# 0.3.2 (2018-11-03)

* Fix regression introduced in 0.3.1: create config path parent subdirectory
  before trying to write inside it.

# 0.3.1 (2018-11-03)

* Add a `dmenv` subdirectory to the configuration file path.

# 0.3.0 (2018-11-01)

* Replace command `freeze` by `lock`.

# 0.2.3 (2018-11-01)

* Add command `dmenv init` to generate a working `setup.py` file.

# 0.2.2 (2018-11-01)

* Fix running dmenv on Windows.
* The configuration file is now read from $HOME (`~/.config` on Linux and macOS, `%HOME%\AppData\Local` on Windows).

# 0.2.1 (2018-10-31)

* The `.dmenv.toml` file is now required.

# 0.2.0 (2018-10-31)

* Can be used with multiple python versions, using the `.dmenv.toml` config file.

# 0.1.0 (2018-10-31)

* Initial release.
