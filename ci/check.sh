#!/bin/bash

set -x
set -e

cargo_audit() {
  cargo install cargo-audit --force
  cargo audit
}

clippy() {
  rustup component add clippy
  cargo clippy --all-targets -- -D warnings
}

build_release() {
  cargo build --release
}

test_release() {
  case $TRAVIS_OS_NAME in
    windows)
      # Make sure Python installed by choco is first in PATH
      export PATH=/c/Python37:${PATH}
      ;;
    linux)
      # Do not use -m venv (buggy on Debian)
      export DMENV_NO_VENV_STDLIB=1
      ;;
    esac
    cargo test --release
}

main() {
  # Run the non-OS specific checks on the fastest platform: linux
  if [[ "$TRAVIS_OS_NAME" == "linux" ]]; then
      cargo_audit
      clippy
  fi

  build_release
  test_release
}

main
