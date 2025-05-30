# Changelog

All notable changes to this project will be documented in this file.

## [0.2.14] - 2025-05-15

### Features

- Add Python 3.14 (beta) support in [#94](https://github.com/PyO3/python3-dll-a/pull/94)

## [0.2.13] - 2025-02-16

### Features

- Add PyPy 3.11 support in [#87](https://github.com/PyO3/python3-dll-a/pull/87)

## [0.2.12] - 2024-12-19

### Features

- Add Python 3.13t support in [#84](https://github.com/PyO3/python3-dll-a/pull/84)

## [0.2.11] - 2024-11-30

### Features

- Add Python 3.13t support in [#82](https://github.com/PyO3/python3-dll-a/pull/82)

## [0.2.10] - 2024-06-24

### Features

- Add Python 3.13 support in [#72](https://github.com/PyO3/python3-dll-a/pull/72)

## [0.2.9] - 2023-07-04

### Fixes

- Fix PyPy 3.10 support in[#46](https://github.com/PyO3/python3-dll-a/pull/46)

## [0.2.8] - 2023-07-03

### Features

- Add PyPy 3.10 support in [#44](https://github.com/PyO3/python3-dll-a/pull/44)

## [0.2.7] - 2023-05-25

### Features

- Add Python 3.12 support in [#34](https://github.com/PyO3/python3-dll-a/pull/34)

## [0.2.6] - 2022-08-21

### Features

- Add MinGW-w64 `dlltool` program name configuration env var [#31](https://github.com/PyO3/python3-dll-a/pull/31)

## [0.2.5] - 2022-07-14

### Fixes

- Fix PyPy import library name in [#27](https://github.com/PyO3/python3-dll-a/pull/27)

## [0.2.4] - 2022-07-14

### Features

- Add PyPy support in [#25](https://github.com/PyO3/python3-dll-a/pull/25)

## [0.2.3] - 2022-05-17

### Features

- Add `zig dlltool` support in [#18](https://github.com/pyo3/python3-dll-a/pull/18)

### Fixes

- Improve error message when `dlltool` is not found in [#17](https://github.com/pyo3/python3-dll-a/pull/17)

## [0.2.2] - 2022-05-10

### Features

- Include `python3.def` itself in the Rust source in [#10](https://github.com/pyo3/python3-dll-a/pull/10)
- Add support for generating non-abi3 `pythonXY.dll` in [#15](https://github.com/pyo3/python3-dll-a/pull/15)

### CI

- Automate `stable_abi.txt` updates in [#6](https://github.com/pyo3/python3-dll-a/pull/6)

## [0.2.1] - 2022-04-17

### Features

- Add support for `lib.exe` from MSVC when running on Windows in [#2](https://github.com/pyo3/python3-dll-a/pull/2)

### Documentation

- Mention the new PyO3 integration feature
- Add maintenance section to README

### Miscellaneous Tasks

- Update stable_abi.txt to the latest main

### CI

- Add `rust.yml` workflow to build and run unit tests
- Add `publish.yml` workflow to publish the crate to `crates.io`

## [0.2.0] - 2022-03-21

### Features

- [**breaking**] Use `Path` type for the output directory arg

## [0.1.2] - 2022-03-15

### Documentation

- Document MSVC ABI environment support

### Features

- Add support for the LLVM `dlltool` flavor

### Testing

- Build import libraries for all targets

## [0.1.1] - 2022-03-14

### Documentation

- Add multi-arch `build.rs` examples

### Features

- Add support for the 32-bit target architecture

## [0.1.0] - 2022-02-21

### Documentation

- Add `build.rs` usage examples

### Features

- Generate module definition and invoke dlltool
- Implement Module-Definition file writing
- Implement 'stable_abi.txt' syntax parser

### Miscellaneous Tasks

- Add `git-cliff` config file
- Add a change log file

<!-- generated by git-cliff -->
