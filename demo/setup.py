import sys
from setuptools import setup, find_packages

if sys.version_info.major < 3:
    sys.exit("Error: Please upgrade to Python3")


setup(
    name="dmenv-demo",
    version="0.6.1",
    description="dmenv demo",
    author="Kontrol SAS",
    py_modules=["dmenv"],
    include_package_data=True,
    install_requires=[
        "path.py",
    ],
    extras_require={
        "dev": [
            "pytest",
        ]
    },
    classifiers=[
        "Programming Language :: Python :: 3.3",
        "Programming Language :: Python :: 3.4",
        "Programming Language :: Python :: 3.5",
        "Programming Language :: Python :: 3.6",
    ],
    entry_points={"console_scripts": ["dmenv-demo = dmenv:main"]},
)
