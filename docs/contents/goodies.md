# dmenv goodies

`dmenv` also comes with a few commands for carrying out boring tasks:

## dmenv run

You can use:`dmenv run` to run any binary from the virtual environment. If the program you want to run
needs command-line options, use a `--` separator, like so:

```console
dmenv run -- pytest --collect-only
```

## dmenv upgrade-pip

Tired of `pip` telling you to upgrade itself? Run `dmenv upgrade-pip` :)

It's exactly the same as typing `dmenv run -- python -m pip install --upgrade pip`, but with less keystrokes :P


## dmenv show:venv_path

Useful to activate the virtual environment for your current shell. For instance:

```bash
source "$(dmenv show:venv_path)/bin/activate"
```

## dmenv show:deps

Just a wrapper for `pip list`:

```bash
$ dmenv show:deps
Package            Version
------------------ -------
atomicwrites       1.2.1
attrs              18.2.0
importlib-metadata 0.6
...
```

## dmenv bump-in-lock

You can use `bump-in-lock` to bump versions directly in the `requirements.lock` file:

```text
# contents of requirements.lock:
bar==0.3
foo==1.2

$ dmenv bump-in-lock bar 0.4

# new contents:
bar==0.4
foo==1.2
```

If you used a git URL in the `requirements.lock` file, you can also bump the git reference:

```text
# contents of requirements.lock:
bar==0.3
foo==git@gitlab.com/foo/foo@master#egg=foo

$ dmenv bump-in-lock --git foo deadbeef
bar==0.3
foo==git@gitlab.com/foo/foo@deadbeef#egg=foo
```
