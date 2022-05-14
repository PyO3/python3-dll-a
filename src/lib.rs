//! Standalone `python3(y).dll` import library generator
//! ====================================================
//!
//! Generates import libraries for the Python DLL
//! (either `python3.dll` or `python3y.dll`)
//! for MinGW-w64 and MSVC (cross-)compile targets.
//!
//! See <https://docs.python.org/3/c-api/stable.html> for the Stable ABI details.
//!
//! This crate **does not require** Python 3 distribution files
//! to be present on the (cross-)compile host system.
//!
//! **Note:** MSVC cross-compile targets require LLVM binutils
//! to be available on the host system.
//! More specifically, `python3-dll-a` requires `llvm-dlltool` executable
//! to be present in `PATH` when targeting `*-pc-windows-msvc` from Linux.
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

use std::ffi::OsStr;
use std::fs::{create_dir_all, write};
use std::io::{Error, ErrorKind, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Import library file extension for the GNU environment ABI (MinGW-w64)
const IMPLIB_EXT_GNU: &str = ".dll.a";

/// Import library file extension for the MSVC environment ABI
const IMPLIB_EXT_MSVC: &str = ".lib";

/// Canonical MinGW-w64 `dlltool` program name
const DLLTOOL_GNU: &str = "x86_64-w64-mingw32-dlltool";

/// Canonical MinGW-w64 `dlltool` program name (32-bit version)
const DLLTOOL_GNU_32: &str = "i686-w64-mingw32-dlltool";

/// Canonical `dlltool` program name for the MSVC environment ABI (LLVM dlltool)
const DLLTOOL_MSVC: &str = "llvm-dlltool";

/// Canonical `lib` program name for the MSVC environment ABI (MSVC lib.exe)
const LIB_MSVC: &str = "lib.exe";

/// Windows import library generator for Python
///
/// Generates `python3.dll` or `pythonXY.dll` import library directly from the
/// embedded Python ABI definitions data for the specified compile target.
#[derive(Debug, Clone)]
pub struct ImportLibraryGenerator {
    /// The compile target architecture name (as in `CARGO_CFG_TARGET_ARCH`)
    arch: String,
    // The compile target environment ABI name (as in `CARGO_CFG_TARGET_ENV`)
    env: String,
    /// Major and minor Python version (for `pythonXY.dll` only)
    version: Option<(u8, u8)>,
}

impl ImportLibraryGenerator {
    /// Creates a new import library generator for the specified compile target.
    ///
    /// The compile target architecture name (as in `CARGO_CFG_TARGET_ARCH`)
    /// is passed in `arch`.
    ///
    /// The compile target environment ABI name (as in `CARGO_CFG_TARGET_ENV`)
    /// is passed in `env`.
    pub fn new(arch: &str, env: &str) -> Self {
        Self {
            arch: arch.to_string(),
            env: env.to_string(),
            version: None,
        }
    }

    /// Sets major and minor version for the `pythonXY.dll` import library.
    ///
    /// The version-agnostic `python3.dll` is generated by default.
    pub fn version(&mut self, version: Option<(u8, u8)>) -> &mut Self {
        self.version = version;
        self
    }

    /// Generates the Python DLL import library in `out_dir`.
    ///
    /// The version-agnostic `python3.dll` import library is generated
    /// by default unless the version-specific `pythonXY.dll` import
    /// was requested via `version()`.
    pub fn generate(&self, out_dir: &Path) -> Result<()> {
        create_dir_all(out_dir)?;

        let defpath = self.write_def_file(out_dir)?;
        let implib_file = self.implib_file_path(out_dir);

        // Try to guess the `dlltool` executable name from the target triple.
        let mut command = match (self.arch.as_str(), self.env.as_str()) {
            // 64-bit MinGW-w64 (aka x86_64-pc-windows-gnu)
            ("x86_64", "gnu") => self.build_dlltool_command(DLLTOOL_GNU, &implib_file, &defpath),
            // 32-bit MinGW-w64 (aka i686-pc-windows-gnu)
            ("x86", "gnu") => self.build_dlltool_command(DLLTOOL_GNU_32, &implib_file, &defpath),
            // MSVC ABI (multiarch)
            (_, "msvc") => {
                if let Some(command) = find_lib_exe(&self.arch) {
                    self.build_dlltool_command(command.get_program(), &implib_file, &defpath)
                } else {
                    self.build_dlltool_command(DLLTOOL_MSVC, &implib_file, &defpath)
                }
            }
            _ => {
                let msg = format!(
                    "Unsupported target arch '{}' or env ABI '{}'",
                    self.arch, self.env
                );
                return Err(Error::new(ErrorKind::Other, msg));
            }
        };

        // Run the selected `dlltool` executable to generate the import library.
        let status = command.status().map_err(|e| {
            let msg = format!("{:?} failed with {}", command, e);
            Error::new(e.kind(), msg)
        })?;

        if status.success() {
            Ok(())
        } else {
            let msg = format!("{:?} failed with {}", command, status);
            Err(Error::new(ErrorKind::Other, msg))
        }
    }

    /// Writes out the embedded Python library definitions file to `out_dir`.
    ///
    /// Returns the newly created `python3.def` or `pythonXY.def` file path.
    fn write_def_file(&self, out_dir: &Path) -> Result<PathBuf> {
        let (def_file, def_file_content) = match self.version {
            None => ("python3.def", include_str!("python3.def")),
            Some((3, 7)) => ("python37.def", include_str!("python37.def")),
            Some((3, 8)) => ("python38.def", include_str!("python38.def")),
            Some((3, 9)) => ("python39.def", include_str!("python39.def")),
            Some((3, 10)) => ("python310.def", include_str!("python310.def")),
            Some((3, 11)) => ("python311.def", include_str!("python311.def")),
            _ => return Err(Error::new(ErrorKind::Other, "Unsupported Python version")),
        };

        let mut defpath = out_dir.to_owned();
        defpath.push(def_file);

        write(&defpath, def_file_content)?;

        Ok(defpath)
    }

    /// Builds the generated import library file name.
    ///
    /// Returns the full import library file path under `out_dir`.
    fn implib_file_path(&self, out_dir: &Path) -> PathBuf {
        let libext = if self.env == "msvc" {
            IMPLIB_EXT_MSVC
        } else {
            IMPLIB_EXT_GNU
        };

        let libname = match self.version {
            Some((major, minor)) => {
                format!("python{}{}{}", major, minor, libext)
            }
            None => format!("python3{}", libext),
        };

        let mut libpath = out_dir.to_owned();
        libpath.push(libname);

        libpath
    }

    /// Generates the complete `dlltool` executable invocation command.
    ///
    /// Supports Visual Studio `lib.exe`, LLVM and MinGW `dlltool` flavors.
    fn build_dlltool_command(
        &self,
        dlltool: impl AsRef<OsStr>,
        libpath: &Path,
        defpath: &Path,
    ) -> Command {
        let dlltool = dlltool.as_ref();
        let mut command = if self.env == "msvc" {
            find_lib_exe(&self.arch).unwrap_or_else(|| Command::new(dlltool))
        } else {
            Command::new(dlltool)
        };

        // Check whether we are using LLVM `dlltool` or MinGW `dlltool`.
        if dlltool == DLLTOOL_MSVC {
            // LLVM tools use their own target architecture names...
            let machine = match self.arch.as_str() {
                "x86_64" => "i386:x86-64",
                "x86" => "i386",
                "aarch64" => "arm64",
                arch => arch,
            };

            command
                .arg("-m")
                .arg(machine)
                .arg("-d")
                .arg(defpath)
                .arg("-l")
                .arg(libpath);
        } else if Path::new(dlltool).file_name() == Some(LIB_MSVC.as_ref()) {
            // lib.exe use their own target architecure names...
            let machine = match self.arch.as_str() {
                "x86_64" => "X64",
                "x86" => "X86",
                "aarch64" => "ARM64",
                arch => arch,
            };
            command
                .arg(format!("/MACHINE:{}", machine))
                .arg(format!("/DEF:{}", defpath.display()))
                .arg(format!("/OUT:{}", libpath.display()));
        } else {
            command
                .arg("--input-def")
                .arg(defpath)
                .arg("--output-lib")
                .arg(libpath);
        }

        command
    }
}

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
    ImportLibraryGenerator::new(arch, env).generate(out_dir)
}

/// Finds Visual Studio `lib.exe` when running on Windows.
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

        for minor in 7..=11 {
            ImportLibraryGenerator::new("x86_64", "gnu")
                .version(Some((3, minor)))
                .generate(&dir)
                .unwrap();
        }
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
        for minor in 7..=11 {
            ImportLibraryGenerator::new("x86_64", "msvc")
                .version(Some((3, minor)))
                .generate(&dir)
                .unwrap();
        }
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
