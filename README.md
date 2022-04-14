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

PyO3 integration
----------------

Since version **0.16.4**, the `pyo3` crate implements support
for the Stable ABI Python DLL import library generation via
its new `generate-abi3-import-lib` feature.

In this configuration, `python3-dll-a` becomes a `pyo3` crate dependency
and is automatically invoked by its build script in both native
and cross compilation scenarios.

### Example `Cargo.toml` usage for a PyO3 extension module

```toml
[dependencies]
pyo3 = { version = "0.16.4", features = ["extension-module", "abi3-py37", "generate-abi3-import-lib"] }
```

Standalone build script usage
-----------------------------

If an older `pyo3` crate version is used, or a different Python bindings
library is required, `python3-dll-a` can be used directly
from the crate build script.

The examples below assume using an older version of PyO3.

### Example `build.rs` script

The following cargo build script can be used to cross-compile Stable ABI
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

### Example `cargo build` invocation

```sh
PYO3_CROSS_LIB_DIR=target/python3-dll cargo build --target x86_64-pc-windows-gnu
```
