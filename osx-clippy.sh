#!/usr/bin/env bash

set -eou pipefail

PATH="/usr/local/darwin-ndk-x86_64/bin/:$PATH" \
CC=o64-clang \
CXX=o64-clang++ \
cargo clippy --target x86_64-apple-darwin

