.PHONY: build
build:
	cargo build
	rm -rf lua/*
	cp target/debug/libpeek.dylib lua/peek.so
	cp -r target/debug/deps lua
test:
	cargo build
	nvim -u test.lua --headless
