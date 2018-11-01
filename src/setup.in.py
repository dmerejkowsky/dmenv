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
    # extension;
    # py_modules=["<module>"],
    install_requires=[
        # Put your dependencies here
        # "path.py"
    ],
    extras_require={
        "dev": [
            # Put you dev dependencies here
            # "pytest"
        ]
    },
    classifiers=[
        # Put the list of supported Python versions here:
        # "Programming Language :: Python :: 3.3",
        # "Programming Language :: Python :: 3.4",
        # ...
    ],
    entry_points={
        "console_scripts":
        # If you are writing an application (and not a library), add its name
        # and the path to the main() function here:
        [
            # "<name> = <package.module:main>",
        ]
    },
)
