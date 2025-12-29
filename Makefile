# Makefile for gwt

PLAYGROUND_IMAGE := gwt-playground

.PHONY: help
help: ## Show this help message
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-20s\033[0m %s\n", $$1, $$2}'

.PHONY: install
install: ## Build and install gwt
	@echo "Building release version of gwt..."
	@cargo install --path .
	@echo "Done!"

.PHONY: setup-pre-commit
setup-pre-commit: ## Install pre-commit hooks
	pre-commit install

.PHONY: playground-build
playground-build: ## Build the playground container image
	docker build -t $(PLAYGROUND_IMAGE) -f Dockerfile.playground .

.PHONY: playground
playground: playground-build ## Start an interactive playground environment in a container
	docker run -it --rm $(PLAYGROUND_IMAGE)