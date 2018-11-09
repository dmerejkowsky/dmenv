import sys
from setuptools import setup, find_packages

if sys.version_info.major < 3:
    sys.exit("Error: Please upgrade to Python3")


setup(
    name="dmenv-docs",
    version="0.1.0",
    description="dmenv documentation",
    author="Dimitri Merejkowsky",
    packages=find_packages(),
    install_requires=[
        "mkdocs",
        "mkdocs-alabaster",
        "ghp-import",
    ],
)
