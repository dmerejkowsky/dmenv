# dmenv: the stupid virtualenv manager

## Usage

Start with a `setup.py` like this:

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

Now you can add `requirements.lock` to your git repo, and anyone can run `dmenv install` to install all the deps.

As a convenience, you can use:

* `dmenv run` to run any binary from the virtualenv
* something like `source $(dmenv show)` to activate the virtualenv for your current shell

# FAQ

Q: How do I add dependencies to build the documentation?<br/>
A: Stick them in the `dev` section.

Q: What if I don't want to install the dev dependencies?<br/>
A: Don't use dmenv. Run `pip install` without `[dev]` extras.


## Why?

* Because pipenv, poetry and the like are too big and too complex
* Because virtualenv + requirements.txt has worked for 10 years and will continue to work for 10 years
* Because it will continue to work if / when pip supports pipfile
* Because dependency management is very hard, and pip already does a good enough job

## Why Python3 only?

* Because it's 2018

## Why not use virtualenv?

* Because python3 -m venv works since Python3.3, except on debian where you have to run `apt install python3-venv`. But that's Debian's problem, not mine

## But I don't want to maintain a `setup.py`!

Too bad. Don't use dmenv, then.

## Why Rust?

* Because I want to make to **never depend** on pip, setuptools or any other internals of pip and virtualenv
* Because it has excellent support for what we need: manipuate paths and run commands in a cross-platform way
* Because it's my second favorite language
* Because distribution is really easy
