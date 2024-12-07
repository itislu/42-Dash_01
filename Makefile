all:
	cargo build
	mv target/debug/marvin .

.PHONY: all
