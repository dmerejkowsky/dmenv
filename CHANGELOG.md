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
