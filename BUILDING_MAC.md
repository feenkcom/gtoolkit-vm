# Mac build

## Prerequisites
There are a couple of required build tools that are needed to build an app.
### Rust
The client side is written in Rust. It can be installed via `rustup`:
```
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```
### CMake
Is responsible for managing the build process of C/C++ third party libraries.
It can be installed directly from [cmake.org](cmake.org/download) or via [brew.sh](brew.sh).
```
brew install cmake
```

