# Install Coffee

> Coffee is under development so currently there is no way to fetch
> just the binary for the architecture, but in the future this will
> be the standard way to install coffee.

## Binary Installation

You can install coffee as a binary tool, but for now
you need to install also [Rust](https://www.rust-lang.org/tools/install)
on your system.

When you had Rust up and running, to install Coffee you need just to do

```bash
git clone https://github.com/coffee-tools/coffee.git && cd coffee
make install
coffee --help
```

You can also install `coffee_httpd` binary to [run coffee as a server](../src/using-coffee.md#running-coffee-as-a-server). run:
```bash
cargo install --bin coffee_httpd
```
