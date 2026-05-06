# Changelog

All notable changes to this project will be documented in this file.

## [Unreleased] - 2024-05-06

### Added
- **`CompiledRule` API:** A new native object that parses JSON rules once and allows high-performance repeated evaluations. Exposed in Rust, Python, and WASM.
- **Strict Batch Variants:** Added `evaluate_batch_strict` and `evaluate_batch_numeric_strict` to fail fast on errors, complementing the default fault-tolerant batch methods.
- **Compliance Suite:** Integrated full JSONLogic specification compliance tests and Property-Based Testing (fuzzing) using `proptest`.
- **Examples:** Added `fraud_scoring.rs`, `feature_flags.rs`, `credit_eligibility.py`, and a full Vite-based `browser_wasm_demo/`.
- **Documentation:** Added `BENCHMARKS.md`, `COMPATIBILITY.md`, `SECURITY.md`, and completely rewrote `README.md` for production readiness.
