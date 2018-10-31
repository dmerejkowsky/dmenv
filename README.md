# dmenv: the stupid virtualenv manager

## Basic usage


`dmenv` only needs one config file: the `.dmenv.toml`.

The config **must** contain a `default` config, like this:

```
[env.default]
python = /path/to/python
```

Note: do *not* put the `.dmenv.toml` under version control, you never know what people install where :)

Then, make sure to have `setup.py` file looking like this:

```python
setup(
  name="foo",
  version="0.1",
  install_requires=[
    # Your deps here
    "bar",
    "baz >= 2.0",
  ],
  extras_require = {
    # Your dev deps here
    "dev": [
      "pytest",
    ]
  },
  entry_points={
     "console_scripts": [
        "foo = foo:main"
      ]
  },
)
```

Run `dmenv freeze`: it will

* Create a virtualenv for you with `python -m venv`
* Run `pip intall --editable .[dev]` so that your dev deps are installed, and the scripts listed in `entry_points` are
  created
* Run `pip freeze` to generate a `requirements.lock` file.

Now you can add `requirements.lock` to your git repo, and then anyone can run `dmenv install` to install all the deps and get exactly the same versions you got when you ran `dmenv freeze`. Hooray reproducible builds!

As a convenience, you can use:

* `dmenv run` to run any binary from the virtualenv
* something like `source $(dmenv show)` to activate the virtualenv for your current shell

## Using an other python version

To use a different Python, version add a new section in `.dmenv` with the name and the path to the binary, like this:

```toml
[env.3.8]
python = "/path/to/python3.8"
```
Then you can use all the `dmenv` commands by prefixing them with `dmenv --env 3.8`.

Cool, no?



# FAQ

Q: How do I add dependencies to build the documentation?<br/>
A: Stick them in the `dev` section.

Q: What if I don't want to install the dev dependencies?<br/>
A: Don't use dmenv. Run `pip install` without `[dev]` extras.

Q: How do I upgrade a dependency?<br/>
A: Just run `dmenv freeze` again. If something breaks, either fix your code or use more precise version specifiers

Q: How do I depend on a git specific repo/branch?<br/>
A: Edit the `requirements.lock` by hand like this:

```
foo==0.1
https://gitlab.com/foo/bar@my-branch
```

Q: But that sucks and it will disappear when I re-run `dmenv freeze`! <br />
A: Yes that sucks. Feel free to:
  * Open a pull request if you've forked an upstream project
  * Use a local pipy mirror and a little bit of CI to publish your sources there



## Why?

* Because pipenv, poetry and tox are too big and too complex
* Because virtualenv + requirements.txt has worked for 10 years and will continue to work for 10 years
* Because it will continue to work if / when pip supports pipfile
* Because dependency management is very hard, and pip already does a good enough job

## Why Python3 only?

* Because it's 2018

## Why not use virtualenv?

* Because python3 -m venv works since Python3.3, except on debian where you have to run `apt install python3-venv`. But that's Debian's problem, not mine

## But I don't want to maintain a `setup.py`!

Too bad. Don't use dmenv, then. poetry is cool.

## Why Rust?

* Because I want to make to **never depend** on pip, setuptools or any other internals of pip and virtualenv
* Because it has excellent support for what we need: manipuate paths and run commands in a cross-platform way
* Because it's my second favorite language
* Because distribution is really easy
