# Changelog

All notable changes to this project will be documented in this file.

## [Unreleased]

## [0.2.0] - 2026-08-06

### Added
- **`CompiledRule` API:** A new native object that parses JSON rules once and allows high-performance repeated evaluations. Exposed in Rust, Python, and WASM.
- **Strict Batch Variants:** Added `evaluate_batch_strict` and `evaluate_batch_numeric_strict` to fail fast on errors, complementing the default fault-tolerant batch methods.
- **Compliance Suite:** Integrated full JSONLogic specification compliance tests and Property-Based Testing (fuzzing) using `proptest`.
- **Examples:** Added `fraud_scoring.rs`, `feature_flags.rs`, `credit_eligibility.py`, and a full Vite-based `browser_wasm_demo/`.
- **Documentation:** Added `BENCHMARKS.md`, `COMPATIBILITY.md`, `SECURITY.md`, and completely rewrote `README.md` for production readiness.

### Changed
- **Dependency Upgrades:**
  - Upgraded `rayon` from `1.11` to `1.12`.
  - Upgraded `thiserror` from `1.0` to `2.0` (breaking update under SemVer).
  - Upgraded `criterion` from `0.5` to `0.8`.
