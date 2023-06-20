CC=cargo
FMT=fmt

ARGS="--all"

default: fmt
	$(CC) build
	@make example

doc-deps:
	$(CC) install mdbook

fmt:
	$(CC) fmt --all

check:
	$(CC) test $(ARGS)

example:
	@echo "No example for the moment"

clean:
	$(CC) clean

book:
	cd docs/docs-book; mdbook build

dev-book:
	cd docs/docs-book; mdbook serve --open

install:
	$(CC) install --locked --path ./coffee_cmd

integration:
	cd tests; $(CC) test $(ARGS)
