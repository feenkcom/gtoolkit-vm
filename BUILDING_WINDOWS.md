# Windows build

## Prerequisites
There are a couple of required build tools that are needed to build an app.

### Chocolatey (optional)
It is a Windows package manager which makes it easier to install tools from the powershell
Follow instructions on [chocolatey.org](https://chocolatey.org/install).

### Microsoft C++ Build Tools 2019 + CLang
Install [Microsoft C++ Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) and choose Native Desktop Development workload.

Then install `C++ Clang tools for Windows` in individual components. Do **not** select `C++ Clang-cl for v142 build tools` if you want Visual Studio to install LLVM + CLang.

In order to build `cairo` install `Windows Universal CRT SDK` from the Individual Components.

The following installs the `Microsoft C++ Build Tools` installer:
```
choco install visualstudio2019buildtools
```

Make sure the following environmental variables are set:
```
LIBCLANG_PATH = C:\Program Files (x86)\Microsoft Visual Studio\2019\BuildTools\VC\Tools\Llvm\x64\bin
LLVM_HOME = C:\Program Files (x86)\Microsoft Visual Studio\2019\BuildTools\VC\Tools\Llvm\x64
PATH = C:\Program Files (x86)\Microsoft Visual Studio\2019\BuildTools\MSBuild\Current\Bin
```

### Rust
The client side is written in Rust. It can be installed via [rustup for Windows](https://win.rustup.rs/x86_64)

### Git
```
choco install -y git
```

### CMake
```
choco install -y cmake
```
### Pkg-Config
```
choco install -y pkgconfiglite
```

### Python2 (for Skia)
```
choco install -y python2
```

### Coreutils
[Coreutils](https://github.com/uutils/coreutils) is required to build `cairo`, since it relies on Makefiles which in turn requires unix command line utilities such as `mkdir`, `dirname`, `echo` and others.
```
git clone https://github.com/uutils/coreutils.git
cd coreutils
cargo install --path .
```

### Make
Make is required to drive the `cairo` build.
```
choco install -y make
```