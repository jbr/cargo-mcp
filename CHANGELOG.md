# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.0](https://github.com/jbr/cargo-mcp/compare/v0.1.1...v0.2.0) - 2025-07-18

### Added

- add cargo_run and add no_capture to cargo_test
- don't persist cargo env
- [**breaking**] rewrite to use mcplease

### Other

- Merge pull request #13 from jbr/mcplease-rewrite
- cargo clippy
- add a smoke test for tools list

## [0.1.1](https://github.com/jbr/cargo-mcp/compare/v0.1.0...v0.1.1) - 2025-06-10

### Added

- add cargo_clean
- default toolchain
- add toolchain support
- rename env to cargo_env
- add env parameter support to all cargo commands

### Fixed

- stable fighting against nightly clippy
- fmt
- resolve unstable let chain syntax error
- fmt

### Other

- update readme
- extract tools schema to a json file
- enable renovate
- Add .github and license files
