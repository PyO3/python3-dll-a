//! Standalone `python3.dll` import library generator
//! =================================================
//!
//! Generates import libraries for the Stable ABI Python DLL
//! for MinGW-w64 and MSVC (cross-)compile targets.
//!
//! See <https://docs.python.org/3/c-api/stable.html> for details.
//!
//! This crate **does not require** Python 3 distribution files
//! to be present on the (cross-)compile host system.
//!
//! **Note:** MSVC (cross-)compile targets require LLVM binutils
//! to be available on the host system.
//! More specifically, `python3-dll-a` requires `llvm-dlltool` executable
//! to be present in `PATH` when targeting `*-pc-windows-msvc`.
//!
//! PyO3 integration
//! ----------------
//!
//! Since version **0.16.4**, the `pyo3` crate implements support
//! for the Stable ABI Python DLL import library generation via
//! its new `generate-abi3-import-lib` feature.
//!
//! In this configuration, `python3-dll-a` becomes a `pyo3` crate dependency
//! and is automatically invoked by its build script in both native
//! and cross compilation scenarios.
//!
//! ### Example `Cargo.toml` usage for a PyO3 extension module
//!
//! ```toml
//! [dependencies]
//! pyo3 = { version = "0.16.4", features = ["extension-module", "abi3-py37", "generate-abi3-import-lib"] }
//! ```
//!
//! Standalone build script usage
//! -----------------------------
//!
//! If an older `pyo3` crate version is used, or a different Python bindings
//! library is required, `python3-dll-a` can be used directly
//! from the crate build script.
//!
//! The examples below assume using an older version of PyO3.
//!
//! ### Example `build.rs` script
//!
//! The following cargo build script can be used to cross-compile Stable ABI
//! PyO3 extension modules for Windows (64/32-bit x86 or 64-bit ARM)
//! using either MinGW-w64 or MSVC target environment ABI:
//!
//! ```no_run
//! fn main() {
//!     if std::env::var("CARGO_CFG_TARGET_OS").unwrap() == "windows" {
//!         let cross_lib_dir = std::env::var_os("PYO3_CROSS_LIB_DIR")
//!             .expect("PYO3_CROSS_LIB_DIR is not set when cross-compiling");
//!         let arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap();
//!         let env = std::env::var("CARGO_CFG_TARGET_ENV").unwrap();
//!
//!         let libdir = std::path::Path::new(&cross_lib_dir);
//!         python3_dll_a::generate_implib_for_target(libdir, &arch, &env)
//!             .expect("python3.dll import library generator failed");
//!     }
//! }
//! ```
//!
//! A compatible `python3.dll` import library file named `python3.dll.a`
//! or `python3.lib` will be automatically created in the directory
//! pointed by the `PYO3_CROSS_LIB_DIR` environment variable.
//!
//! ### Example `cargo build` invocation
//!
//! ```sh
//! PYO3_CROSS_LIB_DIR=target/python3-dll cargo build --target x86_64-pc-windows-gnu
//! ```

#![deny(missing_docs)]
#![allow(clippy::needless_doctest_main)]

use std::fs::{create_dir_all, write};
use std::io::{Error, ErrorKind, Result};
use std::path::Path;
use std::process::Command;

/// Module-Definition file name for `python3.dll`
const DEF_FILE: &str = "python3.def";

/// Module-Definition file content for `python3.dll`
const DEF_FILE_CONTENT: &[u8] = include_bytes!("python3.def");

/// Canonical `python3.dll` import library file name for the GNU environment ABI (MinGW-w64)
const IMPLIB_FILE_GNU: &str = "python3.dll.a";

/// Canonical `python3.dll` import library file name for the MSVC environment ABI
const IMPLIB_FILE_MSVC: &str = "python3.lib";

/// Canonical MinGW-w64 `dlltool` program name
const DLLTOOL_GNU: &str = "x86_64-w64-mingw32-dlltool";

/// Canonical MinGW-w64 `dlltool` program name (32-bit version)
const DLLTOOL_GNU_32: &str = "i686-w64-mingw32-dlltool";

/// Canonical `dlltool` program name for the MSVC environment ABI (LLVM dlltool)
const DLLTOOL_MSVC: &str = "llvm-dlltool";

/// Canonical `lib` program name for the MSVC environment ABI (MSVC lib.exe)
const LIB_MSVC: &str = "lib.exe";

/// Generates `python3.dll` import library directly from the embedded
/// Python Stable ABI definitions data for the specified compile target.
///
/// The import library file named `python3.dll.a` or `python3.lib` is created
/// in directory `out_dir`.
///
/// The compile target architecture name (as in `CARGO_CFG_TARGET_ARCH`)
/// is passed in `arch`.
///
/// The compile target environment ABI name (as in `CARGO_CFG_TARGET_ENV`)
/// is passed in `env`.
pub fn generate_implib_for_target(out_dir: &Path, arch: &str, env: &str) -> Result<()> {
    create_dir_all(out_dir)?;

    let mut defpath = out_dir.to_owned();
    defpath.push(DEF_FILE);

    write(&defpath, DEF_FILE_CONTENT)?;

    // Try to guess the `dlltool` executable name from the target triple.
    let (command, dlltool) = match (arch, env) {
        // 64-bit MinGW-w64 (aka x86_64-pc-windows-gnu)
        ("x86_64", "gnu") => (Command::new(DLLTOOL_GNU), DLLTOOL_GNU),
        // 32-bit MinGW-w64 (aka i686-pc-windows-gnu)
        ("x86", "gnu") => (Command::new(DLLTOOL_GNU_32), DLLTOOL_GNU_32),
        // MSVC ABI (multiarch)
        (_, "msvc") => {
            if let Some(command) = find_lib_exe(arch) {
                (command, LIB_MSVC)
            } else {
                (Command::new(DLLTOOL_MSVC), DLLTOOL_MSVC)
            }
        }
        _ => {
            let msg = format!("Unsupported target arch '{arch}' or env ABI '{env}'");
            return Err(Error::new(ErrorKind::Other, msg));
        }
    };

    // Run the selected `dlltool` executable to generate the import library.
    let status = build_dlltool_command(command, dlltool, arch, &defpath, out_dir).status()?;

    if status.success() {
        Ok(())
    } else {
        let msg = format!("{dlltool} failed with {status}");
        Err(Error::new(ErrorKind::Other, msg))
    }
}

/// Find Visual Studio lib.exe on Windows
#[cfg(windows)]
fn find_lib_exe(arch: &str) -> Option<Command> {
    let target = match arch {
        "x86_64" => "x86_64-pc-windows-msvc",
        "x86" => "i686-pc-windows-msvc",
        "aarch64" => "aarch64-pc-windows-msvc",
        _ => return None,
    };
    cc::windows_registry::find(target, LIB_MSVC)
}

#[cfg(not(windows))]
fn find_lib_exe(_arch: &str) -> Option<Command> {
    None
}

/// Generates the complete `dlltool` executable invocation command.
///
/// Supports Visual Studio `lib.exe`, LLVM and MinGW `dlltool` flavors.
fn build_dlltool_command(
    mut command: Command,
    dlltool: &str,
    arch: &str,
    defpath: &Path,
    out_dir: &Path,
) -> Command {
    let mut libpath = out_dir.to_owned();

    // Check whether we are using LLVM `dlltool` or MinGW `dlltool`.
    if dlltool == DLLTOOL_MSVC {
        libpath.push(IMPLIB_FILE_MSVC);

        // LLVM tools use their own target architecture names...
        let machine = match arch {
            "x86_64" => "i386:x86-64",
            "x86" => "i386",
            "aarch64" => "arm64",
            _ => arch,
        };

        command
            .arg("-m")
            .arg(machine)
            .arg("-d")
            .arg(defpath)
            .arg("-l")
            .arg(libpath);
    } else if dlltool == LIB_MSVC {
        libpath.push(IMPLIB_FILE_MSVC);

        // lib.exe use their own target architecure names...
        let machine = match arch {
            "x86_64" => "X64",
            "x86" => "X86",
            "aarch64" => "ARM64",
            _ => arch,
        };
        command
            .arg(format!("/MACHINE:{}", machine))
            .arg(format!("/DEF:{}", defpath.display()))
            .arg(format!("/OUT:{}", libpath.display()));
    } else {
        libpath.push(IMPLIB_FILE_GNU);

        command
            .arg("--input-def")
            .arg(defpath)
            .arg("--output-lib")
            .arg(libpath);
    }

    command
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    #[cfg(unix)]
    #[test]
    fn generate() {
        // FIXME: Use "target/<arch>" dirs for temporary files.
        let mut dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        dir.push("target");
        dir.push("x86_64-pc-windows-gnu");
        dir.push("python3-dll");

        generate_implib_for_target(&dir, "x86_64", "gnu").unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn generate_gnu32() {
        let mut dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        dir.push("target");
        dir.push("i686-pc-windows-gnu");
        dir.push("python3-dll");

        generate_implib_for_target(&dir, "x86", "gnu").unwrap();
    }

    #[test]
    fn generate_msvc() {
        let mut dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        dir.push("target");
        dir.push("x86_64-pc-windows-msvc");
        dir.push("python3-dll");

        generate_implib_for_target(&dir, "x86_64", "msvc").unwrap();
    }

    #[test]
    fn generate_msvc32() {
        let mut dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        dir.push("target");
        dir.push("i686-pc-windows-msvc");
        dir.push("python3-dll");

        generate_implib_for_target(&dir, "x86", "msvc").unwrap();
    }

    #[test]
    fn generate_msvc_arm64() {
        let mut dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        dir.push("target");
        dir.push("aarch64-pc-windows-msvc");
        dir.push("python3-dll");

        generate_implib_for_target(&dir, "aarch64", "msvc").unwrap();
    }
}
