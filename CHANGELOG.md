# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic
Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Docker support.

  This is now the preferred way of setting up your system.
  Older installation instructions have been deleted.

  Thanks to Steven Jiang for this.

- Demos directory.

  Demonstration code (typically the same code used in documentation)
  is now in the demos directory.

- FFI support.

  This makes it possible to verify programs that combine
  C and Rust. See `demos/simple/ffi`.

- `std::env::args()` support.

  This makes it possible to verify programs that have
  command line arguments. See `demos/simple/argv`.

### Changed

### Deprecated

### Removed

### Fixed

- Many minor documentation errors
- cargo-verify verbosity control produces more useful output for debugging
  cargo-verify with.



[0.0.2]: https://github.com/project-oak/rust-verification-tools/compare/v0.0.1...v0.0.2
[0.0.1]: https://github.com/project-oak/rust-verification-tools/releases/tag/v0.0.1
