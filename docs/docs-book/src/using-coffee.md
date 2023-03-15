# Using Coffee

Coffee is a plugin manager and development tool for core lightning nodes. It helps automate configuration and installation of plugins, and provides plugin templates for all the official languages.

Coffee is a command line utility that provides access to a wide range of tasks.
## First Configuration

## Add a Plugin Repository
Coffee ensures a high-functioning and secure core by allowing users to select repositories from which to download plugins, and then authorizing the installation of only the desired plugins.
To add a plugin repository, simply run the following command.

```bash
coffee remote add <NAME_OF_THE_REPOSITORY> <URL_OF_THE_REPOSITORY>
```
To remove a plugin repository, simply run the following command.
```bash
coffee remote remove <NAME_OF_THE_REPOSITORY>
```

## Install a Plugin
Congratulations! After adding a repository, Coffee will catalogue it, allowing you to explore all the plugins that can be installed via the CLI. Coffee offers multiple installation strategies that you can select based on your preferences, such as:

### Dynamic installation
To install a plugin dynamically, you simply need to run.
```bash
coffee install -d <NAME_OF_PLUGIN>
```
### Static installation
To install a plugin statically, you simply need to run.
```bash
coffee install <NAME_OF_PLUGIN>
```
## Removing a Plugin
To remove an installed plugin, you simply have to run the following command.
```bash
coffee remove <NAME_OF_PLUGIN>
```
## Upgrade a Plugin
Coffee tightly integrates with git, allowing you to easily upgrade your plugins through the command line interface (CLI). This eliminates the need for tedious tasks such as downloading the latest updates and creating new versions of plugins. To upgrade a plugin, all you need to do is run.
```bash
coffee upgrade <NAME_OF_PLUGIN>
```
or if you wish to upgrade several plugins at once.
```bash
coffee upgrade [LIST_OF_PLUGINS]
```