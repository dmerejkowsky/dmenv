#!/bin/bash
set -x
set -e


install_python3() {
  case $TRAVIS_OS_NAME in
    windows)
      choco install python
      ;;
    mac)
      # Installed by homebrew addon, nothing to do :)
      ;;
    linux)
      # Can't trust anything coming from Debian :/
      curl https://bootstrap.pypa.io/get-pip.py -o get-pip.py
      python3 get-pip.py --user
      python3 -m pip install virtualenv --user
      ;;
    esac
}


main() {
  install_python3
}


main
