# dmenv: simple and practical virtual environment manager for Python

[![crates.io image](https://img.shields.io/crates/v/dmenv.svg)](https://crates.io/crates/dmenv)
[![Test Results](https://github.com/TankerHQ/dmenv/workflows/Run%20tests/badge.svg)](https://github.com/TankerHQ/dmenv)
[![Lint Results](https://github.com/TankerHQ/dmenv/workflows/Run%20linters/badge.svg)](https://github.com/TankerHQ/dmenv)

## What it does

`dmenv` takes care of:

* Creating virtual environments for you: one virtual environment per project and Python version, thus
  enforcing some commonly agreed-upon best practices

* Generating a *lock file* that contains *all the versions* of all your dependencies at
  a given time, so you can have reproducible builds

If does it by:

* reading information about your project in the `setup.py` file and nothing else
* using already existing tools such as `python3 -m venv` and `pip`

Want to try it? Proceed to [installation](installation.md) and [usage](./basic_usage.md).
