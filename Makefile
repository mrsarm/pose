.PHONY : test test-cmd test-all fmt-check lint
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

lint:
	cargo clippy -- -D warnings

fmt-check:
	cargo fmt -- --check

test-cmd: build
	./tests/bats/bin/bats tests/run_test.bats

test-all: lint fmt-check test test-cmd
