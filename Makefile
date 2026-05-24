PREFIX ?= $(HOME)/.local
BINDIR ?= $(PREFIX)/bin
CARGO ?= cargo
CARGO_TARGET_DIR ?= target
STRIP ?= strip
STRIP_FLAGS ?= --strip-unneeded
BUILD_VERSION ?= nightly
BUILD_DATE ?= $(shell date -u +%Y-%m-%d)
RELEASE_BIN := $(CARGO_TARGET_DIR)/release/enigma2-player

.PHONY: setup build test run install clean

setup:
	CARGO_TARGET_DIR="$(CARGO_TARGET_DIR)" "$(CARGO)" fetch

build:
	BUILD_VERSION="$(BUILD_VERSION)" BUILD_DATE="$(BUILD_DATE)" CARGO_TARGET_DIR="$(CARGO_TARGET_DIR)" "$(CARGO)" build --release
	"$(STRIP)" $(STRIP_FLAGS) "$(RELEASE_BIN)"

test:
	BUILD_VERSION="$(BUILD_VERSION)" BUILD_DATE="$(BUILD_DATE)" CARGO_TARGET_DIR="$(CARGO_TARGET_DIR)" "$(CARGO)" test

run: build
	"$(RELEASE_BIN)"

install: build
	install -d "$(BINDIR)"
	install -m 755 "$(RELEASE_BIN)" "$(BINDIR)/enigma2-player"
	"$(STRIP)" $(STRIP_FLAGS) "$(BINDIR)/enigma2-player"

clean:
	rm -rf "$(CARGO_TARGET_DIR)"
