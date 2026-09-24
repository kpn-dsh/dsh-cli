#@formatter:off

DSH_CLI_VERSION ?= 0.10.1

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

APPLE_NOTARY_SESSION_ID := $(session)
APPLE_NOTARY_APP_PASSWORD := $(password)

.PHONY: pre-requisites

help:
	@echo "\033[92mTargets:\033[0m"
	@echo "  \033[34mbuild\033[0m              Builds macos and linux binaries."
	@echo "  \033[34mbuild-linux\033[0m        Builds linux binaries."
	@echo "  \033[34mbuild-macos\033[0m        Builds macos binaries."
	@echo "  \033[34mcheck\033[0m              Check if project is ready to be released."
	@echo "  \033[34mcertificate-check\033[0m  Check if KPN developer certificate is installed."
	@echo "  \033[34mhelp\033[0m               Displays this help text."
	@echo "  \033[34mcodesign\033[0m           Run macOS code signing."
	@echo "  \033[34mcodesign-check\033[0m     Check macOS code signing."
	@echo "  \033[34mnotarize\033[0m           Run macOS code notarize (requires password=APP_SPECIFIC_PASSWORD)."
	@echo "  \033[34mnotarize-check\033[0m     Check whether macOS binary is notarized."
	@echo "  \033[34mnotarize-poll\033[0m      Poll macOS code notarize (requires password=APP_SPECIFIC_PASSWORD and session=NOTARY_SESSION_ID)."
	@echo "  \033[34mpublish\033[0m            Publish crates to cargo.io."
	@echo "  \033[34mrelease\033[0m            Create GitHub release."

check: certificate-check
	$(LINUX_TARGET_CC) --version
	cargo +nightly fmt --check
	cargo clippy --all-features
	cargo test --all-features
	cargo deny check licenses

certificate-check:
ifeq ($(shell security find-identity -v -p codesigning | grep -c $(APPLE_DEV_TEAM_ID)), 0)
	@echo "\033[31mdevelopers certificate not installed\033[0m"
else
	@echo "developers certificate installed"
endif

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
	cargo publish --dry-run
	cargo publish

release:
	gh release create

codesign: certificate-check
	codesign -o runtime -s "$(APPLE_DEV_TEAM_NAME)" $(MACOS_BINARY)

codesign-check:
ifeq ("$(wildcard $(MACOS_BINARY))", "")
	@echo "$(MACOS_BINARY) does not exist"
else ifeq ($(shell codesign -dv --deep --strict $(MACOS_BINARY) 2>&1 >/dev/null | grep -c $(APPLE_DEV_TEAM_ID)), 0)
	@echo "$(MACOS_BINARY) exists but is not signed"
else
	@echo "$(MACOS_BINARY) exists and is signed"
endif

notarize:
ifndef $(APPLE_NOTARY_APP_PASSWORD)
	@echo "\033[31mprovide app specific password: password=APP_SPECIFIC_PASSWORD\033[0m"
else
	zip $(APPLE_BINARIES_ZIP) $(MACOS_BINARY)
	xcrun notarytool submit $(APPLE_BINARIES_ZIP) --apple-id $(APPLE_DEVELOPER_APPLE_ID) --team-id $(APPLE_DEV_TEAM_ID) --password $(APPLE_NOTARY_APP_PASSWORD)
endif

notarize-poll:
ifndef $(APPLE_NOTARY_APP_PASSWORD)
	@echo "\033[31mprovide app specific password: password=APP_SPECIFIC_PASSWORD\033[0m"
else ifndef $(APPLE_NOTARY_SESSION_ID)
	@echo "\033[31mprovide Apple notarize session id: session=NOTARY_SESSION_ID\033[0m"
else
	xcrun notarytool log $(APPLE_NOTARY_SESSION_ID) --apple-id $(APPLE_DEVELOPER_APPLE_ID) --team-id $(APPLE_DEV_TEAM_ID)  --password $(APPLE_NOTARY_APP_PASSWORD)
endif

notarize-check:
	codesign -vvvv -R="notarized" --check-notarization $(MACOS_BINARY)
