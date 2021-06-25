# Linux build

## Prerequisites
There are a couple of required build tools that are needed to build an app.
### Rust
The client side is written in Rust. It can be installed via `rustup`:
```
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```
### CMake
Is responsible for managing the build process of C/C++ third party libraries. Install it according to the Linux distribution.

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
```
cargo run --package vm-builder -- --app-name GlamorousToolkit --identifier com.gtoolkit --release
```