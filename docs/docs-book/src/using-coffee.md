# Using Coffee

Coffee is a plugin manager and development tool for core lightning nodes. It
helps automate configuration and installation of plugins, and provides plugin
templates for all the official languages.

Coffee is a command line utility that provides access to a wide range of tasks.

## First Configuration

> ✅ Implemented

Coffee is able to manage the configuration of your core lightning and all the
plugins connected with it, so in order to use Coffee, the user needs to point
Coffee to the current root of core lightning.

By default the Core Lightning home is stored in the `/home/<user>/.lightning`,
and you can do it with the following command

```bash
coffee setup /home/alice/.lightning
```

Then you will find an include at the end of the config file at
`/home/alice/.lightnig/bitcoin/config`, in case this config file do not exist
Coffee will create it.

```text
include /home/alice/.coffee/testnet/coffee.conf
```

In addition there are the following additional option that you can specify:

- `--network`: by default set to `bitcoin`, but if you want specify the network
that Core Lightning is using, you must ensure that the flag is set to
the correct network.
- `--data-dir`: by default this `/home/<home>/.coffee`, you may want to set
this option if you are looking to specify a different directory for the
Coffee home.

## Add a Plugin Repository

> ✅ Implemented

Coffee ensures a high-functioning and secure core by allowing users to select
repositories from which to download plugins, and then authorizing the
installation of only the desired plugins.

To add a plugin repository, simply run the following command.

```bash
coffee remote add <name repository> <url repository>
```

To remove a plugin repository, simply run the following command.

> ✅ Implemented

```bash
coffee remote rm <NAME_OF_THE_REPOSITORY>
```

To list plugin repositories, simply run the following command.

> ✅ Implemented

```bash
coffee remote list 
```

## Install a Plugin

> ✅ Implemented

Congratulations! After adding a repository, Coffee will catalogue it,
allowing you to explore all the plugins that can be
installed via the CLI. Coffee offers multiple installation strategies
that you can select based on your preferences, such as:

### Dynamic installation

To install a plugin dynamically, you simply need to run.

```bash
coffee install -d <NAME_OF_PLUGIN>
```

### Static installation

> ✅ Implemented

To install a plugin statically, you simply need to run.

```bash
coffee install <NAME_OF_PLUGIN>
```

## Removing a Plugin

> ✅ Implemented

To remove an installed plugin, you simply have to run the following command.

```bash
coffee remove <NAME_OF_PLUGIN>
```

## Upgrade a Plugin

> ⚠️  this feature is under development, see [the tracking issue](https://github.com/coffee-tools/coffee/issues/13)

Coffee tightly integrates with git, allowing you to easily upgrade your plugins through the command line interface (CLI). This eliminates the need for tedious tasks such as downloading the latest updates and creating new versions of plugins. To upgrade a plugin, all you need to do is run.

```bash
coffee upgrade <NAME_OF_PLUGIN>
```

## Listing all the plugins

> ✅ Implemented

```bash
coffee list
```

or to see all the plugins inside the repositories

```bash
coffee list --remotes
```

## Showing the README file of the plugin

> ✅ Implemented

```bash
coffee show <NAME_OF_PLUGIN>
```
