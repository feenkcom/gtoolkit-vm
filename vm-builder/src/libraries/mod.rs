mod cmake_library;
mod library;
mod native_library;
mod openssl_library;
mod rust_library;
mod system_library;

pub use cmake_library::CMakeLibrary;
pub use library::{
    GitLocation as LibraryGitLocation, Library, LibraryLocation,
    PathLocation as LibraryPathLocation,
};
pub use native_library::{NativeLibrary, NativeLibraryDependencies};
pub use openssl_library::OpenSSLLibrary;
pub use rust_library::RustLibrary;

#[cfg(target_os = "windows")]
pub fn git() -> CMakeLibrary {
    let libssh2 = LibraryLocation::Git(
        LibraryGitLocation::new("https://github.com/libssh2/libssh2.git")
            .directory("ssh2")
            .tag("libssh2-1.9.0"),
    );

    let libgit2 = LibraryLocation::Git(
        LibraryGitLocation::new("https://github.com/libgit2/libgit2.git").tag("v1.1.1"),
    );

    CMakeLibrary::new("git2", LibraryLocation::Multiple(vec![libssh2, libgit2]))
        .define("EMBED_SSH_PATH", "../../ssh2")
        .define("BUILD_CLAR", "OFF")
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
pub fn git() -> CMakeLibrary {
    let openssl = OpenSSLLibrary::new();

    let libssh2 = CMakeLibrary::new(
        "ssh2",
        LibraryLocation::Git(
            LibraryGitLocation::new("https://github.com/libssh2/libssh2.git").tag("libssh2-1.9.0"),
        ),
    )
    .depends(Box::new(openssl));

    CMakeLibrary::new(
        "git2",
        LibraryLocation::Git(
            LibraryGitLocation::new("https://github.com/libgit2/libgit2.git").tag("v1.1.1"),
        ),
    )
    .alias("git2.1.1.1")
    .define("CMAKE_SHARED_LINKER_FLAGS:STRING", "-lssl -lcrypto")
    .define("BUILD_CLAR", "OFF")
    .depends(Box::new(libssh2))
}

pub fn sdl2() -> CMakeLibrary {
    CMakeLibrary::new(
        "SDL2",
        LibraryLocation::Git(
            LibraryGitLocation::new("https://github.com/libsdl-org/SDL.git").tag("release-2.0.14"),
        ),
    )
    .alias("SDL2d")
}

pub fn glutin() -> RustLibrary {
    RustLibrary::new(
        "Glutin",
        LibraryLocation::Git(LibraryGitLocation::new(
            "https://github.com/feenkcom/libglutin.git",
        )),
    )
}

pub fn boxer() -> RustLibrary {
    RustLibrary::new(
        "Boxer",
        LibraryLocation::Git(LibraryGitLocation::new(
            "https://github.com/feenkcom/gtoolkit-boxer.git",
        )),
    )
}

pub fn skia() -> RustLibrary {
    RustLibrary::new(
        "Skia",
        LibraryLocation::Git(LibraryGitLocation::new(
            "https://github.com/feenkcom/libskia.git",
        )),
    )
    .requires("python")
}

pub fn gleam() -> RustLibrary {
    RustLibrary::new(
        "Gleam",
        LibraryLocation::Git(LibraryGitLocation::new(
            "https://github.com/feenkcom/libgleam.git",
        )),
    )
}

pub fn winit() -> RustLibrary {
    RustLibrary::new(
        "Winit",
        LibraryLocation::Git(LibraryGitLocation::new(
            "https://github.com/feenkcom/libwinit.git",
        )),
    )
}

pub fn clipboard() -> RustLibrary {
    RustLibrary::new(
        "Clipboard",
        LibraryLocation::Git(LibraryGitLocation::new(
            "https://github.com/feenkcom/libclipboard.git",
        )),
    )
}
