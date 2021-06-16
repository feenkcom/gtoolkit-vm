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

## Building

First build a `vm-builder`:
```
cargo build --bin vm-builder --release
```
Then build an app:
```
./target/release/vm-builder --release --app-name GlamorousToolkit --identifier com.gtoolkit -vv
```
The bundled app will be placed in the following folder within `gtoolkit-vm`:
```
target/{architecture}/release/bundle/
```

To see all possible options of the `vm-builder`:
```
./target/release/vm-builder --help
```

### Dependencies
The client side is written in Rust. It can be installed via `rustup`:
```
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```
For Windows, it is necessary to install [Microsoft C++ Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/).

### Compiling for Apple M1

Install the corresponding rust target and toolchain
```
rustup target add aarch64-apple-darwin
rustup toolchain install stable-aarch64-apple-darwin
```