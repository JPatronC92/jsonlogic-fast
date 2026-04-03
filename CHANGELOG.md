# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] — Unreleased

### Added

- Core JSON-Logic evaluator (`jsonlogic-fast` crate)
- Single and batch evaluation with parallel processing (Rayon)
- Numeric coercion and type extraction utilities
- Python bindings via PyO3 (`jsonlogic-fast-python`)
- WASM bindings via wasm-bindgen (`jsonlogic-fast-wasm`)
- Criterion benchmarks (single, generic, batch 10K)
- GitHub Actions CI (fmt, clippy, test, audit, deny, Python e2e, WASM runtime, bench smoke)
- `cargo-deny` policy for license and advisory checks
- 55 tests: 11 Rust core + 29 Python e2e + 15 WASM runtime
- Dual MIT/Apache-2.0 license
