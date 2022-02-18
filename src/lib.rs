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

#![deny(missing_docs)]

use std::fs::create_dir_all;
use std::fs::File;
use std::io::{BufWriter, Error, ErrorKind, Result, Write};
use std::path::PathBuf;
use std::process::Command;

/// Stable ABI Python DLL file name
const DLL_FILE: &str = "python3.dll";

/// Canonical `python3.dll` import library file name for MinGW-w64
const IMPLIB_FILE: &str = "python3.dll.a";

/// Module-Definition file name for `python3.dll`
const DEF_FILE: &str = "python3.def";

/// Canonical MinGW-w64 `dlltool` program name
const DLLTOOL: &str = "x86_64-w64-mingw32-dlltool";

/// Python Stable ABI symbol defs from the CPython repository
///
/// Upstream source: <https://github.com/python/cpython/blob/main/Misc/stable_abi.txt>
const STABLE_ABI_DEFS: &str = include_str!("../Misc/stable_abi.txt");

/// Generates `python3.dll` import library directly from the embedded
/// Python Stable ABI definitions data.
///
/// The import library file named `python3.dll.a` is created
/// in directory `out_dir`.
pub fn generate_implib(out_dir: &str) -> Result<()> {
    create_dir_all(out_dir)?;

    let mut libpath = PathBuf::from(out_dir);
    let mut defpath = libpath.clone();

    libpath.push(IMPLIB_FILE);
    defpath.push(DEF_FILE);

    let stable_abi_exports = parse_stable_abi_defs(STABLE_ABI_DEFS);

    let mut writer = BufWriter::new(File::create(&defpath)?);
    write_export_defs(&mut writer, DLL_FILE, &stable_abi_exports)?;
    drop(writer);

    let status = Command::new(DLLTOOL)
        .arg("--input-def")
        .arg(defpath)
        .arg("--output-lib")
        .arg(libpath)
        .status()?;

    if status.success() {
        Ok(())
    } else {
        let msg = format!("{DLLTOOL} failed with {status}");
        Err(Error::new(ErrorKind::Other, msg))
    }
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
