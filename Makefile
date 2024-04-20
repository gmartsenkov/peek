.PHONY: build
test:
	cargo build
	nvim -u test.lua --headless

