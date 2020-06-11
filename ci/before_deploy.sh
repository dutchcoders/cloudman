#!/bin/bash

# All files which should be added only if they changed
aux_files=()

# Output binary name
NAME="cloudman-$TRAVIS_TAG-$TARGET"

# Everything in this directory will be offered as download for the release
mkdir "./target/deploy"

mkdir $NAME
cp target/$TARGET/release/cloudman $NAME/
cp README.md LICENSE $NAME/
tar czvf $NAME.tar.gz $NAME
