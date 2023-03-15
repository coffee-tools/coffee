CC=cargo
FMT=fmt

OPTIONS=

doc-deps:
	$(CC) install mdbook

default: fmt
	$(CC) build
	@make example

fmt:
	$(CC) fmt --all

check:
	$(CC) test --all

example:
	@echo "No example for the moment"

clean:
	$(CC) clean

book:
	cd docs/docs-book; mdbook build

dev-book:
	cd docs/docs-book; mdbook serve --open
