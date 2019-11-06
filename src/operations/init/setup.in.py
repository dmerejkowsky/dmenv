import sys
from setuptools import setup, find_packages

if sys.version_info.major < 3:
    sys.exit("Error: Please upgrade to Python3")


setup(
    name="<NAME>",
    version="<VERSION>",
    description="<DESCRIPTION>",
    author="<AUTHOR>",
    packages=find_packages(),
    # If you have just one file, remove the line above
    # and add it in the list below, *without* the .py
    # extension:
    # py_modules=["<module>"],
    install_requires=[
        # Insert common dependencies here:
        # "colorama",
    ],
    extras_require={
        "dev": [
            # Insert dev dependencies here
            # "pytest",
        ],
        "prod": [
            # Insert prod dependencies here
            # "gunicorn",
        ]
    },
    classifiers=[
        # Insert the list of supported Python versions here:
        # ...
        # "Programming Language :: Python :: 3.5",
        # "Programming Language :: Python :: 3.6",
        # "Programming Language :: Python :: 3.7",
    ],
    entry_points={
        "console_scripts":
        # If you are writing a command line application,
        # add its name and the path to the main() function here:
        [
            # "<name> = <package.module:main>",
        ]
    },
)
