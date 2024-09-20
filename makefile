# Packages to publish in dependency order
PACKAGES := dsh-api trifonius-engine
# Ignore "rus-version" only grep "version" in Cargo.toml
VERSION = $(shell grep -E "^version\s*="  Cargo.toml | head -n 1 | cut -d '"' -f 2)

build:
	@echo "Building $*"
	cargo build

login:
	@echo "Login to KPN Artifactory"
	@echo "https://artifacts.kpn.org/ui/repos/tree/General/cargo-dsh-iuc-local"
	@echo
	@echo "Generate a token by pressing 'Set Me Up' (make sure you are logged in)" 
	@read -p "Enter your token: " token; \
	token_lower=$$(echo "$$token" | tr '[:upper:]' '[:lower:]'); \
	if [ "$${token_lower:0:6}" != "bearer" ]; then \
		token="Bearer $$token"; \
	fi; \
	cargo login --registry artifactory "$$token"

publish-all:
	@echo "Publishing all packages with version: $(VERSION)"
	@for package in $(PACKAGES); do \
		echo "Publishing $$package"; \
		cargo publish -p $$package --registry artifactory; \
	done


publish-all-allow-dirty:
	@echo "Publishing all packages with version: $(VERSION)"
	@for package in $(PACKAGES); do \
		echo "Publishing $$package"; \
		cargo publish -p $$package --registry artifactory --allow-dirty; \
	done

publish-package-%:
	cargo publish -p $* --registry artifactory

publish-allow-dirty-%:
	cargo publish -p $* --registry artifactory --allow-dirty

publish-dry-run-%:
	cargo publish -p $* --registry artifactory--dry-run

test:
	cargo test

help:
	@echo "Targets Cargo:"
	@echo "  build:                         Build all cargo packages"
	@echo "  login:                         Login to KPN Artifactory for the cargo registry"
	@echo "  publish-all:                   Publish all cargo packages to KPN Artifactory"
	@echo "  publish-all-allow-dirty:       Publish all cargo packages to KPN Artifactory without checking for uncommited files"
	@echo "  publish-<package>:             Publish a single cargo package to KPN Artifactory"
	@echo "  publish-allow-dirty-<package>: Publish a single cargo package to KPN Artifactory without checking for uncommited files"
	@echo "  test:                          Run all cargo tests"
	@echo "  test-<package>:                Run tests for a single cargo package"
