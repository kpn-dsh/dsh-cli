#@formatter:off



#


# > rustup target add x86_64-unknown-linux-musl
# > brew install filosottile/musl-cross/musl-cross

DSH_CLI_VERSION ?= 0.10.0

RELEASE_DIR := releases

LINUX_TARGET_CC := x86_64-linux-musl-gcc
LINUX_TARGET_TRIPLE := x86_64-unknown-linux-musl
LINUX_BINARY := $(RELEASE_DIR)/dsh-v$(DSH_CLI_VERSION)-$(LINUX_TARGET_TRIPLE)

MACOS_TARGET_TRIPLE := aarch64-apple-darwin
MACOS_BINARY := $(RELEASE_DIR)/dsh-v$(DSH_CLI_VERSION)-$(MACOS_TARGET_TRIPLE)

APPLE_DEVELOPER_APPLE_ID ?= wilbert.schelvis@kpn.com
APPLE_DEV_TEAM_ID ?= B86ZND72C8
APPLE_DEV_TEAM_NAME ?= Developer ID Application: KPN B.V. (B86ZND72C8)
APPLE_BINARIES_ZIP := $(RELEASE_DIR)/dsh-v$(DSH_CLI_VERSION)-mac-binaries.zip

APPLE_NOTARY_SESSION_ID := session
APPLE_NOTARY_APP_PASSWORD := password

.PHONY: pre-requisites

help:
	@echo "\033[92mTargets:\033[0m"
	@echo "  \033[34mbuild\033[0m           Builds macos and linux binaries."
	@echo "  \033[34mbuild-linux\033[0m     Builds linux binaries."
	@echo "  \033[34mbuild-macos\033[0m     Builds macos binaries."
	@echo "  \033[34mcheck\033[0m           Check if project is ready to be released."
	@echo "  \033[34mhelp\033[0m            Displays this help text."
	@echo "  \033[34mcodesign\033[0m        Run macOS code signing."
	@echo "  \033[34mcodesign-check\033[0m  Check macOS code signing."
	@echo "  \033[34mnotarize\033[0m        Run macOS code notarize (requires password=APP_SPECIFIC_PASSWORD)."
	@echo "  \033[34mnotarize-check\033[0m  Check macOS code notarizing."
	@echo "  \033[34mnotarize-poll\033[0m   Poll macOS code notarize (requires password=APP_SPECIFIC_PASSWORD and session=NOTARY_SESSION_ID)."
	@echo "  \033[34mpublish\033[0m         Publish crates to cargo.io."
	@echo "  \033[34mrelease\033[0m         Create GitHub release."

check:
	$(LINUX_TARGET_CC) --version
	cargo +nightly fmt --check
	cargo clippy
	cargo deny check licenses

pre-requisites: check
	mkdir -p $(RELEASE_DIR)

build: pre-requisites build-macos build-linux

build-linux:
	TARGET_CC=$(LINUX_TARGET_CC) cargo build --release --target $(LINUX_TARGET_TRIPLE)
	mv target/$(LINUX_TARGET_TRIPLE)/release/dsh $(LINUX_BINARY)

build-macos:
	cargo build --release
	mv target/release/dsh $(MACOS_BINARY)

publish:
	# check Cargo.toml for proper dsh_api dependencies
	cargo publish --dry-run
	cargo publish

release:
	gh release create

codesign:
	codesign -o runtime -s $(APPLE_DEV_TEAM_NAME) $(MACOS_BINARY)

codesign-check:
	codesign -vvv --deep --strict $(MACOS_BINARY)

notarize:
	zip $(APPLE_BINARIES_ZIP) $(MACOS_BINARY)
	xcrun notarytool submit $(APPLE_BINARIES_ZIP) --apple-id $(APPLE_DEVELOPER_APPLE_ID) --team-id $(APPLE_DEV_TEAM_ID) --password $(APPLE_NOTARY_APP_PASSWORD)

notarize-poll:
	xcrun notarytool log $(APPLE_NOTARY_SESSION_ID) --apple-id $(APPLE_DEVELOPER_APPLE_ID) --team-id $(APPLE_DEV_TEAM_ID)  --password $(APPLE_NOTARY_APP_PASSWORD)

notarize-check:
	codesign -vvvv -R="notarized" --check-notarization $(MACOS_BINARY)
