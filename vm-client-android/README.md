## Android

The frontend of the app differs when targeting Android. Compared to desktop distributions,
Android orchestrates apps via its Java Activities. There is no concept of `main()` function,
that is executed when an app start up. Instead, `NativeActivity` calls native function via ffi.
Therefor the frontend should be compiled and distributed as a shared library.

Build for android without creating apk:
```bash
cargo apk -- build --package vm-client-android -vvvv
```