# Makefile for gwt

# Default installation path for macOS
INSTALL_PATH ?= /usr/local/bin
BINARY_NAME = gwtree

.PHONY: help
help: ## Show this help message
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-20s\033[0m %s\n", $$1, $$2}'

.PHONY: build
build: ## Build debug version
	@echo "Building debug version of $(BINARY_NAME)..."
	@cargo build

.PHONY: release
release: ## Build release version
	@echo "Building release version of $(BINARY_NAME)..."
	@cargo build --release

.PHONY: install
install: release ## Build and install to INSTALL_PATH (default: /usr/local/bin)
	@echo "Installing $(BINARY_NAME) at $(INSTALL_PATH)/$(BINARY_NAME)"
	@mkdir -p $(INSTALL_PATH)
	@cp target/release/$(BINARY_NAME) $(INSTALL_PATH)/$(BINARY_NAME)
	@chmod +x $(INSTALL_PATH)/$(BINARY_NAME)
	@echo "Done! Run '$(BINARY_NAME) --help' to get started."

.PHONY: uninstall
uninstall: ## Remove installed binary from INSTALL_PATH
	@echo "Removing $(BINARY_NAME) from $(INSTALL_PATH)"
	@rm -f $(INSTALL_PATH)/$(BINARY_NAME)
	@echo "Done!"

.PHONY: clean
clean: ## Clean build artifacts
	@echo "Cleaning build artifacts..."
	@cargo clean

.PHONY: test
test: ## Run tests
	@cargo test

.PHONY: setup-pre-commit
setup-pre-commit: ## Install pre-commit hooks
	pre-commit install