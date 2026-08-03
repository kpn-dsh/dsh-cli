#@formatter:off

# > rustup target add x86_64-unknown-linux-musl
# > brew install filosottile/musl-cross/musl-cross

VERSION ?= 0.10.0

RELEASE_DIR := releases

LINUX_TARGET_TRIPLE := x86_64-unknown-linux-musl
MACOS_TARGET_TRIPLE := aarch64-apple-darwin

.PHONY: pre-requisites


help:
	@echo "\033[92mTargets:\033[0m"
	@echo "  \033[34mbuild\033[0m        Builds macos and linux binaries."
	@echo "  \033[34mbuild-macos\033[0m  Builds macos binaries."
	@echo "  \033[34mbuild-linux\033[0m  Builds linux binaries."
	@echo "  \033[34mhelp\033[0m         Displays this help text."

pre-requisites:
	mkdir -p $(RELEASE_DIR)
	x86_64-linux-musl-gcc --version

build: build-macos build-linux

build-macos: pre-requisites
	cargo build --release
	mv target/release/dsh $(RELEASE_DIR)/dsh-v$(VERSION)-$(MACOS_TARGET_TRIPLE)
	cargo build --all-features --release
	mv target/release/dsh $(RELEASE_DIR)/dsh-manage-v$(VERSION)-$(MACOS_TARGET_TRIPLE)

build-linux: pre-requisites
	TARGET_CC=x86_64-linux-musl-gcc cargo build --release --target $(LINUX_TARGET_TRIPLE)
	mv target/$(LINUX_TARGET_TRIPLE)/release/dsh $(RELEASE_DIR)/dsh-v$(VERSION)-$(LINUX_TARGET_TRIPLE)
	TARGET_CC=x86_64-linux-musl-gcc cargo build --all-features --release --target $(LINUX_TARGET_TRIPLE)
	mv target/$(LINUX_TARGET_TRIPLE)/release/dsh $(RELEASE_DIR)/dsh-manage-v$(VERSION)-$(LINUX_TARGET_TRIPLE)

