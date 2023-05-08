SHELL=/usr/bin/env bash

.PHONY: all
all: 
ifeq ($(OS),Windows_NT)
	$(MAKE) windows
else
	$(MAKE) linux
endif

.PHONY: clean
clean: 
ifeq ($(OS),Windows_NT)
	del /f /q git-* git_* libgit_*.rlib
else
	rm -f git-{compare,csv,meta} libgit_*.rlib
endif

.PHONY: linux
linux: git-compare git-csv git-meta

.PHONY: windows
windows: git-compare.exe git-csv.exe git-meta.exe

git-compare git-csv git-meta:
	cargo build --release --out-dir . -Z unstable-options

git-compare.exe git-csv.exe git-meta.exe:
	cargo build --release --out-dir . -Z unstable-options
