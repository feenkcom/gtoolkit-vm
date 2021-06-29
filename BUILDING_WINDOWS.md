# Windows build

## Prerequisites
There are a couple of required build tools that are needed to build an app.

### Chocolatey (optional)
It is a Windows package manager which makes it easier to install tools from the powershell
Follow instructions on [chocolatey.org](https://chocolatey.org/install).

### Microsoft C++ Build Tools 2019 + CLang
Install [Microsoft C++ Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) and choose Native Desktop Development workload with CLang support.

### Rust
The client side is written in Rust. It can be installed via `rustup`:
```
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### CMake
```
choco install cmake
```
### Pkg-Config
```
choco install pkgconfiglite
```