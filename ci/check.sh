#!/bin/bash

set -x
set -e

cargo_audit() {
  cargo install cargo-audit --force
  cargo audit
}

clippy() {
  rustup component add clippy
  cargo clippy --all-targets -- --deny warnings
}

fmt_check() {
  rustup component add rustfmt
  cargo fmt -- --check
}

build() {
  cargo build --release
}

run_tests() {
  export RUST_BACKTRACE=1

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


  if [[ "${TRAVIS_RUST_VERSION}" == "stable" ]] && [[ "${TRAVIS_OS_NAME}" == "linux" ]]; then
    cargo install cargo-tarpaulin --force
    cargo tarpaulin --ignore-tests --out Xml
    bash <(curl -s https://codecov.io/bash)
  else
    cargo test --release
  fi

 }

main() {
  if [[ "${TRAVIS_OS_NAME}" == "linux" ]] && [[ "${TRAVIS_RUST_VERSION}" == "stable" ]]; then
    # Run the non-OS specific checks on the fastest platform: linux
    fmt_check
    cargo_audit
    clippy
  fi

  build

  run_tests
}

main
