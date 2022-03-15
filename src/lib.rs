//! Standalone `python3.dll` import library generator
//! =================================================
//!
//! Generates import libraries for the Stable ABI Python DLL
//! for MinGW-w64 cross-compile targets.
//!
//! See <https://docs.python.org/3/c-api/stable.html> for details.
//!
//! This crate **does not require** Python 3 distribution files
//! to be present on the cross-compile host system.
//!
//! Example `build.rs` script
//! -------------------------
//!
//! The following script can be used to cross-compile Stable ABI
//! PyO3 extension modules for Windows (64-bit):
//!
//! ```no_run
//! fn main() {
//!     if std::env::var("TARGET").unwrap() == "x86_64-pc-windows-gnu" {
//!         let libdir = std::env::var("PYO3_CROSS_LIB_DIR")
//!             .expect("PYO3_CROSS_LIB_DIR is not set when cross-compiling");
//!         python3_dll_a::generate_implib(&libdir)
//!             .expect("python3.dll import library generator failed");
//!     }
//! }
//! ```
//!
//! A compatible `python3.dll` import library will be automatically created in
//! the directory pointed by `PYO3_CROSS_LIB_DIR` environment variable.
//!
//! If both 64-bit and 32-bit Windows cross-compile targets support is needed,
//! the more generic `generate_implib_for_target()` function must be used:
//!
//! ```no_run
//! fn main() {
//!     if std::env::var("CARGO_CFG_TARGET_OS").unwrap() == "windows"
//!         && std::env::var("CARGO_CFG_TARGET_ENV").unwrap() == "gnu"
//!     {
//!         let libdir = std::env::var("PYO3_CROSS_LIB_DIR")
//!             .expect("PYO3_CROSS_LIB_DIR is not set when cross-compiling");
//!         let arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap();
//!         python3_dll_a::generate_implib_for_target(&libdir, &arch, "gnu")
//!             .expect("python3.dll import library generator failed");
//!     }
//! }
//! ```
//!
//! Example `cargo build` invocation
//! --------------------------------
//!
//! ```sh
//! PYO3_CROSS_LIB_DIR=target/python3-dll cargo build --target x86_64-pc-windows-gnu
//! ```

#![deny(missing_docs)]
#![allow(clippy::needless_doctest_main)]

use std::fs::create_dir_all;
use std::fs::File;
use std::io::{BufWriter, Error, ErrorKind, Result, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Stable ABI Python DLL file name
const DLL_FILE: &str = "python3.dll";

/// Module-Definition file name for `python3.dll`
const DEF_FILE: &str = "python3.def";

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

/// Python Stable ABI symbol defs from the CPython repository
///
/// Upstream source: <https://github.com/python/cpython/blob/main/Misc/stable_abi.txt>
const STABLE_ABI_DEFS: &str = include_str!("../Misc/stable_abi.txt");

/// Generates `python3.dll` import library directly from the embedded
/// Python Stable ABI definitions data for the specified compile target.
///
/// The import library file named `python3.dll.a` is created
/// in directory `out_dir`.
///
/// The compile target architecture name (as in `CARGO_CFG_TARGET_ARCH`)
/// is passed in `arch`.
///
/// The compile target environment ABI name (as in `CARGO_CFG_TARGET_ENV`)
/// is passed in `env`.
pub fn generate_implib_for_target(out_dir: &str, arch: &str, env: &str) -> Result<()> {
    create_dir_all(out_dir)?;

    let mut defpath = PathBuf::from(out_dir);
    defpath.push(DEF_FILE);

    let stable_abi_exports = parse_stable_abi_defs(STABLE_ABI_DEFS);

    let mut writer = BufWriter::new(File::create(&defpath)?);
    write_export_defs(&mut writer, DLL_FILE, &stable_abi_exports)?;
    drop(writer);

    // Try to guess the `dlltool` executable name from the target triple.
    let dlltool = match (arch, env) {
        // 64-bit MinGW-w64 (aka x86_64-pc-windows-gnu)
        ("x86_64", "gnu") => DLLTOOL_GNU,
        // 32-bit MinGW-w64 (aka i686-pc-windows-gnu)
        ("x86", "gnu") => DLLTOOL_GNU_32,
        // MSVC ABI (multiarch)
        (_, "msvc") => DLLTOOL_MSVC,
        _ => {
            let msg = format!("Unsupported target arch '{arch}' or env ABI '{env}'");
            return Err(Error::new(ErrorKind::Other, msg));
        }
    };

    // Run the selected `dlltool` executable to generate the import library.
    let status = build_dlltool_command(dlltool, arch, &defpath, out_dir).status()?;

    if status.success() {
        Ok(())
    } else {
        let msg = format!("{dlltool} failed with {status}");
        Err(Error::new(ErrorKind::Other, msg))
    }
}

/// Generates the complete `dlltool` executable invocation command.
///
/// Supports both LLVM and MinGW `dlltool` flavors.
fn build_dlltool_command(dlltool: &str, arch: &str, defpath: &Path, out_dir: &str) -> Command {
    let mut libpath = PathBuf::from(out_dir);
    let mut command = Command::new(dlltool);

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

/// Generates `python3.dll` import library directly from the embedded
/// Python Stable ABI definitions data for the default 64-bit MinGW-w64
/// compile target.
///
/// The import library file named `python3.dll.a` is created
/// in directory `out_dir`.
///
/// The import library is generated for the default `x86_64-pc-windows-gnu`
/// cross-compile target.
pub fn generate_implib(out_dir: &str) -> Result<()> {
    generate_implib_for_target(out_dir, "x86_64", "gnu")
}

/// Exported DLL symbol definition
struct DllExport {
    /// Export symbol name
    symbol: String,
    /// Data symbol flag
    is_data: bool,
}

/// Parses 'stable_abi.txt' export symbol definitions
fn parse_stable_abi_defs(defs: &str) -> Vec<DllExport> {
    // Try to estimate the number of records from the file size.
    let mut exports = Vec::with_capacity(defs.len() / 32);

    for line in defs.lines() {
        let is_data = if line.starts_with("function") {
            false
        } else if line.starts_with("data") {
            true
        } else {
            // Skip everything but "function" and "data" entries.
            continue;
        };

        // Parse "function|data PyFoo"-like strings.
        if let Some(name) = line.split_ascii_whitespace().nth(1) {
            let symbol = name.to_owned();
            exports.push(DllExport { symbol, is_data })
        }
    }

    exports
}

/// Writes Module-Definition file export statements.
///
/// The library module name is passed in `dll_name`,
/// the list of exported symbols - in `exports`.
///
/// See <https://docs.microsoft.com/en-us/cpp/build/reference/module-definition-dot-def-files>.
fn write_export_defs(writer: &mut impl Write, dll_name: &str, exports: &[DllExport]) -> Result<()> {
    writeln!(writer, "LIBRARY \"{dll_name}\"")?;
    writeln!(writer, "EXPORTS")?;

    for e in exports {
        if e.is_data {
            writeln!(writer, "{} DATA", e.symbol)?;
        } else {
            writeln!(writer, "{}", e.symbol)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate() {
        // FIXME: Use "target/test" dir for temporary files.
        let mut dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        dir.push("target");
        dir.push("test");

        let out_dir = dir.to_str().unwrap();
        generate_implib(out_dir).unwrap();
    }

    #[test]
    fn abi_defs_len() {
        assert_eq!(STABLE_ABI_DEFS.len(), 48836);
    }

    #[test]
    fn parse_stable_abi_txt() {
        let stable_abi_exports = parse_stable_abi_defs(STABLE_ABI_DEFS);

        assert_eq!(stable_abi_exports.len(), 857);
        // assert_eq!(stable_abi_exports.capacity(), 1526);

        let data_sym_num = stable_abi_exports.iter().filter(|x| x.is_data).count();
        assert_eq!(data_sym_num, 143);

        assert_eq!(stable_abi_exports[0].symbol, "PyType_FromSpec");
        assert!(!stable_abi_exports[0].is_data);

        assert_eq!(stable_abi_exports[200].symbol, "PyExc_UnicodeDecodeError");
        assert!(stable_abi_exports[200].is_data);
    }

    #[test]
    fn write_exports() {
        let function = DllExport {
            symbol: "foo".to_owned(),
            is_data: false,
        };
        let data = DllExport {
            symbol: "buf".to_owned(),
            is_data: true,
        };
        let exports = vec![function, data];

        let mut writer = Vec::new();
        write_export_defs(&mut writer, DLL_FILE, &exports).unwrap();

        assert_eq!(
            String::from_utf8(writer).unwrap(),
            "LIBRARY \"python3.dll\"\nEXPORTS\nfoo\nbuf DATA\n"
        );
    }
}
