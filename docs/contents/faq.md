# FAQ

#### I'm on Debian, and I've got errors when running `bdist_wheel`

This is an [upstream bug](https://bugs.debian.org/cgi-bin/bugreport.cgi?bug=917006).
As a workaround, you can install virtualenv with `python3 -m pip install virtualenv --user`
and then set the `DMENV_NO_VENV_STDLIB` environment variable.

#### How do I upgrade dependencies?

Two solutions:

* Patch the `requirements.lock` directly;
* Or run `dmenv clean && dmenv lock` again. If something breaks, either fix your code or
use more precise version specifiers in `setup.py`, like `foobar < 2.0`.



#### How do I depend on a git specific repo/branch?

Edit the `requirements.lock` by hand like this, where the part after `#egg=` matches the name of the dependency in the `setup.py`

```text
https://gitlab.com/foo/bar@my-branch#egg=bar
```

Note that the change will be lost if you re-run `dmenv lock`: see [#7](https://github.com/TankerHQ/dmenv/issues/7)
for details.

We are looking for a proper solution. In the mean time, feel free to:

  * Open a pull request if you've forked an upstream project
  * Use a local pipy mirror and a little bit of CI to publish your sources there

#### How do I use an other Python interpreter?

You can either:

* modify your PATH environment variable so that it appears there. (For instance, with [pyenv](https://github.com/pyenv/pyenv)).
* or prefix all the `dmenv` commands with `--python /path/to/other/python`.


#### Why Rust?

* Because it has excellent support for what we need: manipulate paths and run commands in a cross-platform way
* Because it's my second favorite language
* Because distribution is really easy
* Because by *not* using Python at all `dmenv` is less likely to break if something on your system changes.
