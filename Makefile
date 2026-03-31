.PHONY: install uninstall build

PREFIX ?= /usr/local

build:
	cargo build --release

install: build
	install -d $(PREFIX)/bin
	install -m 755 target/release/demovid $(PREFIX)/bin/demovid

uninstall:
	rm -f $(PREFIX)/bin/demovid
