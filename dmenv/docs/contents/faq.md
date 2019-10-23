# FAQ

#### I'm on Debian, and I've got errors when running `bdist_wheel`

This is an [upstream bug](https://bugs.debian.org/cgi-bin/bugreport.cgi?bug=917006).
As a workaround, you can install virtual environment with `python3 -m pip install virtual environment --user`
and then set the `DMENV_NO_VENV_STDLIB` environment variable to a non-empty value like `1`.

#### Why Rust?

* Because it has excellent support for what we need: manipulate paths and run commands in a cross-platform way
* Because it's my second favorite language
* Because distribution is really easy
* Because by *not* using Python at all `dmenv` is less likely to break if something on your system changes.
