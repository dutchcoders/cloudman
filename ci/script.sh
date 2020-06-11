#!/bin/bash

# Run clippy checks
if [ "$CLIPPY" == "true" ]; then
    rustup component add clippy
    cargo clippy --all-targets
    exit
fi

# Run clippy rustfmt
if [ "$RUSTFMT" == "true" ]; then
    cargo fmt -- --check
    exit
fi

# Run test in release mode if a tag is present, to produce an optimized binary
if [ -n "$TRAVIS_TAG" ]; then
    # Build separately so we generate an 'cloudman' binary without -HASH appended
    cargo build --release --target $TARGET
    cargo test --release
else
    cargo test
fi
