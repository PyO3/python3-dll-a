Standalone `python3.dll` import library generator
=================================================

Generates import libraries for the Stable ABI Python DLL
for MinGW-w64 cross-compile targets.

See <https://docs.python.org/3/c-api/stable.html> for details.

This crate **does not require** Python 3 distribution files
to be present on the cross-compile host system.

Example `build.rs` script
-------------------------

The following script can be used to cross-compile Stable ABI
PyO3 extension modules for Windows:

```rust
fn main() {
    if std::env::var("TARGET").unwrap() == "x86_64-pc-windows-gnu" {
        let libdir = std::env::var("PYO3_CROSS_LIB_DIR")
            .expect("PYO3_CROSS_LIB_DIR is not set when cross-compiling");
        python3_dll_a::generate_implib(&libdir)
            .expect("python3.dll import library generator failed");
    }
}
```

A compatible `python3.dll` import library will be automatically created in
the directory pointed by `PYO3_CROSS_LIB_DIR` environment variable.

If both 64-bit and 32-bit Windows cross-compile targets support is needed,
the more generic `generate_implib_for_target()` function must be used:

```rust
fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap() == "windows"
        && std::env::var("CARGO_CFG_TARGET_ENV").unwrap() == "gnu"
    {
        let libdir = std::env::var("PYO3_CROSS_LIB_DIR")
            .expect("PYO3_CROSS_LIB_DIR is not set when cross-compiling");
        let arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap();
        python3_dll_a::generate_implib_for_target(&libdir, &arch, "gnu")
            .expect("python3.dll import library generator failed");
    }
}
```

Example `cargo build` invocation
--------------------------------

```sh
PYO3_CROSS_LIB_DIR=target/python3-dll cargo build --target x86_64-pc-windows-gnu
```
