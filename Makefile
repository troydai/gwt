# Makefile for gwt

.PHONY: help
help: ## Show this help message
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-20s\033[0m %s\n", $$1, $$2}'

.PHONY: setup-pre-commit
setup-pre-commit: ## Install pre-commit hooks
	pre-commit install

.PHONY: check
check: ## Run all pre-commit hooks on all files
	pre-commit run --all-files

.PHONY: build
build: ## Build the project
	cargo build

.PHONY: test
test: ## Run tests
	cargo test

.PHONY: fmt
fmt: ## Format code
	cargo fmt --all

.PHONY: clippy
clippy: ## Run clippy linter
	cargo clippy -- -D warnings
