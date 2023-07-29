CC=cargo
FMT=fmt

ARGS=""

default: fmt
	$(CC) build

doc-deps:
	$(CC) install mdbook

fmt:
	$(CC) fmt --all

check: ## Runs unit testing
	$(CC) test $(ARGS)

clean: ## Clean up everythings
	$(CC) clean

book: ## Build the release version of documentation
	cd docs/docs-book; mdbook build

dev-book: ## Build the docs in dev mode
	cd docs/docs-book; mdbook serve --open

install: ## Install coffee inside the local machine
	$(CC) install --locked --path ./coffee_cmd

integration: default ## Runs integration testing
	$(CC) test -j 4 -p tests $(ARGS)

setup:
	git config core.hooksPath .githooks

help: ## Show Help
	@grep --no-filename -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | \
	awk 'BEGIN {FS = ":.*?## "}; {printf "\033[32m%-15s\033[0m %s\n", $$1, $$2}'
