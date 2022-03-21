Standalone `python3.dll` import library generator
=================================================

Generates import libraries for the Stable ABI Python DLL
for MinGW-w64 and MSVC (cross-)compile targets.

See <https://docs.python.org/3/c-api/stable.html> for details.

This crate **does not require** Python 3 distribution files
to be present on the (cross-)compile host system.

**Note:** MSVC (cross-)compile targets require LLVM binutils
to be available on the host system.
More specifically, `python3-dll-a` requires `llvm-dlltool` executable
to be present in `PATH` when targeting `*-pc-windows-msvc`.

Example `build.rs` script
-------------------------

The following Cargo build script can be used to cross-compile Stable ABI
PyO3 extension modules for Windows (64/32-bit x86 or 64-bit ARM)
using either MinGW-w64 or MSVC target environment ABI:

```rust
fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap() == "windows" {
        let cross_lib_dir = std::env::var_os("PYO3_CROSS_LIB_DIR")
            .expect("PYO3_CROSS_LIB_DIR is not set when cross-compiling");
        let arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap();
        let env = std::env::var("CARGO_CFG_TARGET_ENV").unwrap();

        let libdir = std::path::Path::new(&cross_lib_dir);
        python3_dll_a::generate_implib_for_target(libdir, &arch, &env)
            .expect("python3.dll import library generator failed");
    }
}
```

A compatible `python3.dll` import library file named `python3.dll.a`
or `python3.lib` will be automatically created in the directory
pointed by the `PYO3_CROSS_LIB_DIR` environment variable.

Example `cargo build` invocation
--------------------------------

```sh
PYO3_CROSS_LIB_DIR=target/python3-dll cargo build --target x86_64-pc-windows-gnu
```
