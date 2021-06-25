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

### GTK
Is required by the [nativefiledialog](https://github.com/saurvs/nfd-rs) to open an image picker. Install it according to the Linux distribution. In case of Ubuntu:
```
sudo apt install libgtk-3-dev
```

### LLVM / CLang
The native libraries are such as Skia and Pharo are compiled using CLang. Install it according to the Linux distribution. In case of Ubuntu:
```
sudo apt install clang llvm lld
```

You may need to override the default compiler and linker:
```
sudo ln -sf /usr/bin/clang /usr/bin/cc
sudo ln -sf /usr/bin/clang++ /usr/bin/c++
sudo ln -sf /usr/bin/clang-cpp /usr/bin/cpp
sudo ln -sf /usr/bin/ld.lld /usr/bin/ld
```

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