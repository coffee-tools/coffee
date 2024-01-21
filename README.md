# Coffee Plugin Manager

Welcome to **Coffee**, the plugin manager for Core Lightning. 
It takes care of all the configuration and installation of a plugin for your Core Lightning node. 

Using Coffee, anyone can manage plugins for their Core Lightning node. 
Coffee handles all the tedious setup and configuration tasks of the underlying 
Core Lightning plugin infrastructure and the hassle of setting up dependencies.

Coffee helps you keep your plugins up to date. With its powerful CLI and git support, 
you can easily update your plugin to the latest version, ensuring it works seamlessly. 
So, no matter what your plugin needs are, Coffee can help you get up and running in no time.

To learn more visit our [documentation](https://coffee-docs.netlify.app)

## Packages

Coffee offers a range of resources and libraries to simplify integrating lightning plugins into 
your business applications :smile:.


| Package        | Description                                                     | Version    |
|----------------|-----------------------------------------------------------------|------------|
| [coffee_core](/coffee_core/)     | Package containing the main implementation of Coffee plugin manager      | pre_release |
| [coffee_cmd](/coffee_cmd/)     | Package providing CLI to the Coffee plugin manager      | pre_release |
| [coffee_github](/coffee_github/)     | GitHub interface to the Coffee plugin manager      | pre_release |
| [coffee_lib](/coffee_lib/)     | The core library to the Coffee plugin ecosystem      | pre_release |
| [coffee_storage](/coffee_storage/)     | The local storage model package for the Coffee plugin manager     | pre_release |
| [coffee_httpd](/coffee_httpd/)     | HTTP daemon that expose the public API of coffee     | under development |
| [coffee_plugin](/coffee_plugin)     | Core Lightning plugin that allow to interact with coffee     | under development |

## How to contribute

Read our [Hacking guide](docs/docs-book/src/contributing-to-coffee.md)

## License

<div align="center">
  <img src="https://opensource.org/wp-content/uploads/2009/06/OSI_Keyhole.svg" width="150" height="200"/>
</div>

```
Copyright 2023 Vincenzo Palazzo <vincenzopalazzodev@gmail.com>. All rights reserved.
```
