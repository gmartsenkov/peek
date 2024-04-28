.PHONY: build
build:
	cargo build --release
	rm -rf lua/*
	cp target/release/libpeek.dylib lua/peek.so
	cp -r target/release/deps lua
test:
	cargo build
	nvim -u test.lua --headless
