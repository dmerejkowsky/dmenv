# dmenv: the stupid virtual environment manager for Python

[![crates.io image](https://img.shields.io/crates/v/dmenv.svg)](https://crates.io/crates/dmenv)
[![Build](https://img.shields.io/travis/TankerHQ/dmenv.svg?branch=master)](https://travis-ci.org/TankerHQ/dmenv)

## What it does

`dmenv` takes care of:

* Creating virtual environments for you: one virtual environment per project and Python version, thus
  enforcing some commonly agreed-upon best practices

* Generating a *lock file* that contains *all the versions* of all your dependencies at
  a given time, so you can have reproducible builds

If does it by:

* reading information about your project in the `setup.py` file and nothing else
* using already existing tools such as `python3 -m venv` and `pip`

As the tag line implies, its implementation is as simple as possible and it contains the
*bare minimum* amount of features.

Want to try it? Proceed to [installation](installation.md) and [usage](./basic_usage.md).
