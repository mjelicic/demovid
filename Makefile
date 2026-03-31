.PHONY: install uninstall build check-deps

PREFIX ?= /usr/local

check-deps:
	@command -v cmake >/dev/null 2>&1 || { \
		echo "error: cmake is required to build demovid"; \
		echo ""; \
		echo "  brew install cmake"; \
		echo ""; \
		exit 1; \
	}

build: check-deps
	cargo build --release

install: build
	install -d $(PREFIX)/bin
	install -m 755 target/release/demovid $(PREFIX)/bin/demovid

uninstall:
	rm -f $(PREFIX)/bin/demovid
