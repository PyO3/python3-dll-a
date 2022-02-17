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

use std::io::Result;

/// Canonical `python3.dll` import library file name for MinGW-w64
const IMPLIB_FILE: &str = "python3.dll.a";

/// Generates `python3.dll` import library directly from the embedded
/// Python Stable ABI definitions data.
///
/// The import library file named `python3.dll.a` is created
/// in directory `out_dir`.
pub fn generate_implib(out_dir: &str) -> Result<()> {
    todo!("writing {IMPLIB_FILE} to {out_dir}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic]
    fn generate() {
        generate_implib("target/lib").unwrap();
    }
}
