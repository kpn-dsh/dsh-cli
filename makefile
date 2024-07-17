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


publish:
	@echo "Publishing trifonius-engine"
	cargo publish -p trifonius-engine --registry artifactory

publish-allow-dirty:
	@echo "Publishing trifonius-engine"
	cargo publish -p trifonius-engine --registry artifactory --allow-dirty

publish-dry-run:
	@echo "Dry-run trifonius-engine"
	cargo publish -p trifonius-engine --registry artifactory --dry-run

test:
	cargo test

help:
	@echo "Targets Cargo:"
	@echo "  build:                 Build all cargo packages"
	@echo "  login:                 Login to KPN Artifactory for the cargo registry"
	@echo "  publish:               Publish to KPN Artifactory"
	@echo "  publish-allow-dirty:   Publish to KPN Artifactory without checking for uncommited files"
	@echo "  publish-dry-run:       Dry-run the publish to KPN Artifactory"
	@echo "  test:                  Run all cargo tests"
	@echo "  test-<package>:        Run tests for a single cargo package"
