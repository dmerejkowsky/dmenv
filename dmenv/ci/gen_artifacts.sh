#!/bin/bash

set -x
set -e


mkdir -p dist/

if [[ "$TRAVIS_OS_NAME" == "windows" ]]; then
  ext=".exe"
else
  ext=""
fi

cp "target/release/dmenv${ext}" "dist/dmenv-${TRAVIS_OS_NAME}${ext}"
