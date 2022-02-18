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

use std::fs::create_dir_all;
use std::fs::File;
use std::io::{Error, ErrorKind, Result, Write};
use std::path::PathBuf;
use std::process::Command;

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

    // TODO: Generate a syntactically valid Module-Definition file.
    File::create(&defpath)?.write_all(STABLE_ABI_DEFS.as_bytes())?;

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
}
