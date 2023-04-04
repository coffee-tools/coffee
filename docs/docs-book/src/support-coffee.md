# How to support?

Core Lightning support many different ways to write a plugin, and if this is bad or good
will not be discussed in this section.
However, we must have a way to install automatically all the plugins that the ecosystem supports.

With Coffee, we choose to give this freedom to the developers of the plugin, in fact, coffee uses a
manifest where it is possible to specify the install script and the runnable file/binary file.

## Add a Manifest file

The coffee manifest is a yaml file with the name `coffee.yml` or `coffee.yaml`, where it is possible
specify the install script an other meta information.

Coffee will heavily search this file in the root directory of the plugin, and it is not so different
from what the other languages like nodejs, go, rust does.

An example of the coffee manifest that will install a Kotlin plugin is the following one:

```yaml
---
plugin:
  name: btcli4j
  version: 0.0.1
  lang: java
  install: |
    sh -C ./gradlew createRunnableScript
  main: btcli4j-gen.sh
```

Where it is possible to specify the following options:

- `name`: the official name of the plugin, that Coffee will referer to during the installation process;
- `version`: the version of the plugin, that currently is not used;
- `lang`: the language of the plugin, used to try to install a plugin when the `install` script is not specified;
- `install`: a custom install script used by Coffee to compile the plugin;
- `main`: the binary or runnable file that core lightning needs to run.

In the future, the coffee will be also able to install `binary` other than a `plugin`, so coffee will be installed with coffee
itself. With some craziness will be also possible to manage core lightning itself.

Please if you feel that additional meta information needs to be specified open an issue 
https://github.com/coffee-tools/coffee/issues
