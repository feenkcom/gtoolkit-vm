# Linux build

## Prerequisites
There are a couple of required build tools that are needed to build an app.
### Rust
The client side is written in Rust. It can be installed via `rustup`:
```
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```
### CMake
it is responsible for managing the build process of C/C++ third party libraries.
Install it according to the Linux distribution.

### GTK
It Is required by the [nativefiledialog](https://github.com/saurvs/nfd-rs) to open an image picker.
Install it according to the Linux distribution. In case of Ubuntu:
```
sudo apt install libgtk-3-dev
```

### SSL
It Is required by multiple Rust crates
```
sudo apt install libssl-dev
```

### LLVM / CLang / Automake / Ninja
The native libraries such as Skia and Pharo are compiled using CLang.
Install it according to the Linux distribution.
In the case of Ubuntu:
```
sudo apt install clang llvm lld autoconf automake libtool libtool-bin ninja-build make build-essential
```

You may need to override the default compiler and linker:
```
sudo ln -sf /usr/bin/clang /usr/bin/cc
sudo ln -sf /usr/bin/clang++ /usr/bin/c++
sudo ln -sf /usr/bin/clang-cpp /usr/bin/cpp
sudo ln -sf /usr/bin/ld.lld /usr/bin/ld
```

### patchelf
The rpaths of the GT executables are updated by gtoolkit-vm-builder:
```
sudo apt install patchelf
```

