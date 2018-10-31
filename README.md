# dmenv: the stupid virtualenv manager

## Why?

* Because pipenv, poetry and the like are too big and too complex
* Because virtualenv + requirements.txt has worked for 10 years and will continue to work for 10 years
* Because it will continue to work if / when pip supports pipfile

## Why Python3 only?

* Because it's 2018

## Why not use virtualenv?

* Because python3 -m venv works since Python3.3, except on debian where you have to run `apt install python3-venv`. But that's Debian's problem, not mine

## Why Rust?

* Because I want to make to **never depend** on pip, setuptools or any other internals of pip and virtualenv
* Because it has excellent support for what we need: manipuate paths and run commands in a cross-platform way
* Because it's my second favorite language
* Because distribution is really easy
