# gtoolkit-vm
Client side Rust implementation of the Glamorous Toolkit VM

## Building

To build a release version of the bundle:
```
cargo run --bin vm-builder -- --release
```

To see all possible options of the `vm-builder`:
```
cargo run --bin vm-builder -- --help
```


### Compiling for Apple M1

Install the corresponding rust target and toolchain
```
rustup target add aarch64-apple-darwin
rustup toolchain install stable-aarch64-apple-darwin
```