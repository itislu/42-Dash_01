NAME := marvin

all:
	cargo build
	rm -f $(NAME)
	cp -rf target/debug/$(NAME) .

.PHONY: all
