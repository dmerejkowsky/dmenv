# dmenv: simple and practical virtualenv manager for Python

[![Docs](https://img.shields.io/badge/docs-latest-green.svg)](https://tankerhq.github.io/dmenv/)
[![crates.io image](https://img.shields.io/crates/v/dmenv.svg)](https://crates.io/crates/dmenv)
[![Build](https://img.shields.io/travis/TankerHQ/dmenv.svg?branch=master)](https://travis-ci.org/TankerHQ/dmenv)
[![Coverage](https://img.shields.io/codecov/c/github/TankerHQ/dmenv.svg?label=Coverage)


## Overview

`dmenv` handles creation of virtualenv and lock files for you.

Here it is in action:

* First, generate a `requirements.lock` to "freeze" all your dependencies

```bash
$ dmenv lock
Creating virtualenv in: /path/to/.venv/3.6.7
-> running /usr/bin/python3 -m /path/to/.venv venv/3.6.7
-> running /path/to/.venv/3.6.7/bin/python -m pip install pip --upgrade
...
-> running /path/to/.venv/3.6.7/bin/pip freeze --exclude-editable
:: Requirements written to /path/to/requirements.lock
```

* Then, anyone can use the `requirements.lock` to install all the dependencies
  at their frozen version:

```bash
$ dmenv install
:: Creating virtualenv in: /path/to/.venv/3.6.7
-> running /usr/bin/python3 -m venv /path/to/.venv/3.6.7
-> running /path/to/.venv/3.6.7/bin/python -m pip install pip --upgrade
...
-> running /path/to/.venv/3.6.7/bin/python setup.py develop --no-deps
...
Installing demo script to /path/to/.venv/3.6.7/bin
```


## Interested?

Go [read the fine documentation](https://tankerhq.github.io/dmenv/) and learn how
to use dmenv for your own Python project :)
