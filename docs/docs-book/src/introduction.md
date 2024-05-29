# What is Coffee? and Why do we need it?

The Coffee plugin manager facilitates the use of the Core Lightning plugin
architecture for both users and developers. It offers assistance to
them while working with Core Lightning.
> At this time, the only true documentation for the plugin manager is the code
itself, and some features that are listed as unsupported in the documentation
may already be supported in the code, as the plugin manager is still in
the process of being developed.

## Problems

Core lightning has a very powerful plugin system that empowers
everyone to write a plugin for core lightning to extend the functionality
of it.

In fact, currently, we have many ways to write a plugin in core lightning
with different languages, like Rust, Go, Java, Kotlin, Scala, Dart, Python
and many others. This can lead to disarray in the community, as the user now
has to follow a different procedure for installing a plugin and building it.

Despite allowing all users to write plugins in their own way, this can have a
detrimental effect in fragmenting the ecosystem.

## Solution

We are offering Coffee as a solution to help install and develop a plugin for
Core Lightning, similar to how Cargo is used for Rust.

In fact, Coffee is implemented to be a modular plugin manager and supports all
the user needs.

Currently, with Coffee it is possible to install a plugin from a GitHub repository
and allow the user to manage it.

In fact, you can install a Java plugin that supports Coffee in a couple of
commands after you [install it](./install-coffee.md)

```bash
coffee --network testnet link /home/alice/.lightning
coffee --network testnet remote add lightningd https://github.com/lightningd/plugins.git
coffee --network testnet install btcli4j
```

>The plugin manager is currently under development, so some of the plugins can
be not supported, but please submit an issue
[here](https://github.com/coffee-tools/coffee/issues) and we try to help
you :smile:

## Ideas

The idea that we have for Coffee is to be the de facto tool to install and
build a plugin in Core Lightning, so we are planning to release it in different
way, such as:

- as a command line tool: empower the developers and the Linux hackers to play with
Coffee from command line;
- as a web server: empower the web and mobile applications to manage Core Lightning;
- as a plugin: empower current systems like gRPC and REST to interact with
with Coffee through the Core Lightning RPC commands.

## Contribute

To empower everyone, we need the help and the vision of everyone, so
if you want to contribute with ideas please submit an
[issue](https://github.com/coffee-tools/coffee/issues) or if you want to
contribute please read our [hacking guide](./contributing-to-coffee.md)

In addition, consider subscribing to [our mailing list](https://lists.sr.ht/~vincenzopalazzo/coffee-dev)
