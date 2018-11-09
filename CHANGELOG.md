# 0.6.0

* Run `setup.py develop` with `--no-deps`
* Rename `show` to `show:venv_path`, add `show:deps` to display the list of dependencies

# 0.5.0

* `dmenv init`: since name is required, it is now an argument, no longer an option.
  So instead of `dmenv init --name foo --version 0.42`, use `dmenv init foo --version 0.42`
* Add a command named `dmenv develop` that justs runs `python setup.py develop` and nothing else
* `dmenv install`: add `--no-upgrade-pip` and `--no-develop` options

# 0.4.3

* Add a `--author` option to `dmenv init`, used when generating the `setup.py` file
* `dmenv lock` now exits immediately if the lock file is missing. (#12)
* Workaround Debian bug in pip (#15)

# 0.4.2

* Write some metadata inside the `requirements.lock` file

* Improve `dmenv run`:
  * Suggest running `lock` or `install`
  * Do not crash if used without arguments


# 0.4.1

* Fix CI on Windows.

# 0.4.0

* `dmenv` no longer needs a configuration file
* Find the Python interpreter to use by looking in the PATH environment variable

# 0.3.4

* If `dmenv` is run *inside an existing virtualenv*, just use it. Fix #9

# 0.3.3

* Also upgrade pip when running `dmenv install`
* Fix incorrect message when running `dmenv lock`

# 0.3.2

* Fix regression introduced in 0.3.1: create config path parent subdirectory
  before trying to write inside it.

# 0.3.1

* Add a `dmenv` subdirectory to the configuration file path.

# 0.3.0

* Replace command `freeze` by `lock`.

# 0.2.3

* Add command `dmenv init` to generate a working `setup.py` file.

# 0.2.2

* Fix running dmenv on Windows
* The configuration file is now read from $HOME (`~/.config` on Linux and macOS, `%HOME%\AppData\Local` on Windows).

# 0.2.1

* The `.dmenv.toml` file is now required.

# 0.2.0

* Can be used with multiple python versions, using the `.dmenv.toml` config file.

# 0.1.0

* Initial release.
