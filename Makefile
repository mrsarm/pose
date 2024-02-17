.PHONY : test test-integration test-cmd test-all-fast test-all fmt-check lint
.DEFAULT_GOAL := release

clean:
	cargo clean

build:
	# Debug binary
	cargo build

release:
	# Release binary at ./target/release/pose
	cargo build --release

install:
	# Binary should be installed at ~/.cargo/bin
	cargo install --path .

test:
	cargo test

test-integration:
	# Run slow tests pose -> docker -> docker registry
	cargo test -- --ignored

lint:
	cargo clippy -- -D warnings

fmt-check:
	cargo fmt -- --check

test-cmd: build
	./tests/bats/bin/bats tests/run_test.bats

test-all-fast: lint fmt-check test test-cmd

test-all: test-all-fast test-integration
