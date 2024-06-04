.PHONY: build
build:
	cargo build --release
	rm -rf lua/*
	[[ -e ./target/release/libpeek.dylib ]] && cp ./target/release/libpeek.dylib ./lua/peek.so || cp ./target/release/libpeek.so ./lua/peek.so
	cp -r target/release/deps lua
test:
	cargo build
	nvim -u test.lua --headless
