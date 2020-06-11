CARGO           = cargo
CARGO_ARGS      =

ROOTDIR         = $(abspath $(dir $(firstword $(MAKEFILE_LIST))))
WORKDIR         = ${ROOTDIR}/.build

DATE = $(shell date +'%Y%m%d%H%M%S')

build-darwin:
	$(CARGO) $(CARGO_ARGS) build --target=x86_64-apple-darwin --release
	cp ./target/x86_64-apple-darwin/release/cloudman $(WORKDIR)/cloudman-darwin-x86_64

build-linux:
	$(CARGO) $(CARGO_ARGS) build --target=x86_64-unknown-linux-gnu --release
	cp ./target/x86_64-unknown-linux-gnu/release/cloudman $(WORKDIR)/cloudman-linux-x86_64

fmt:
	$(CARGO) fmt

run:
	$(CARGO) $(CARGO_ARGS) run

.PHONY: build run
