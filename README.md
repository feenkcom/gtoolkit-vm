# Glamorous Toolkit App
Client side [Glamorous Toolkit](https://github.com/feenkcom/gtoolkit) application written in Rust. It is responsible for handling the command line arguments (using [clap](https://github.com/clap-rs/clap/)) and interactive application launch (such as double-click).

The goal of this project is to provide a cross-platform experience and easy parametrisation of the end-user applications based on the Glamorous Toolkit. The bundled client side app is able to provide runtimes for various scripting languages, such as [Pharo](https://pharo.org), in which at the moment the largest portion of the Glamorous Toolkit is written. The VM for the Pharo language is shipped in a form of a shared library as part of the bundle.

The build process is split in two steps: compilation and packaging. It is orchestrated by the `vm-builder` cli tool.

## Cloning
```
git clone git@github.com:feenkcom/gtoolkit-vm.git
cd gtoolkit-vm
```

After cloning the repository please update the submodules:
```
git submodule update --init --recursive
```

To update a submodule:
```
git submodule update --remote --recursive
```

## Building
The building of the vm happens with the help of `gtoolkit-vm-builder`.

Follow the README.md of [github.com/feenkcom/gtoolkit-vm-builder](https://github.com/feenkcom/gtoolkit-vm-builder).

### Building
Please see a corresponding how-to guide based on the target platform:
 - [Linux](BUILDING_LINUX.md)
 - [MacOS](BUILDING_MAC.md)
 - [Windows](BUILDING_WINDOWS.md)