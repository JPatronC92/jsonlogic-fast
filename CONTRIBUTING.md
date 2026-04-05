# Contributing to jsonlogic-fast

## Prerequisites

- [Rust 1.75+](https://rustup.rs/)
- [uv](https://docs.astral.sh/uv/) (for Python bindings)
- [wasm-pack](https://rustwasm.github.io/wasm-pack/installer/) (for WASM tests)

## Setup

```bash
# Build the Python bindings locally
make setup
```

## Running tests

```bash
make test           # Core Rust tests
make test-python    # Python e2e tests (builds bindings first)
make test-wasm      # WASM runtime tests in Node.js
make ci-local       # Full quality gate (test + clippy + python + wasm + bench)
```

## Running benchmarks

```bash
make bench          # Full Criterion benchmarks (HTML reports in target/criterion/)
make bench-quick    # Reduced sample size for quick feedback
```

## Project structure

| Directory | Description |
|---|---|
| `core/` | Rust engine — all evaluation logic lives here |
| `python/` | PyO3 bindings (compiled with maturin) |
| `wasm/` | wasm-bindgen bindings |
| `tests/python/` | Python e2e tests (pytest) |
| `examples/python/` | Usage examples |

## Code style

- Run `cargo clippy -- -D warnings` before submitting.
- Run `cargo fmt` to format Rust code.
- Add tests for any new functionality.

## License

By contributing, you agree that your contributions will be licensed under the MIT OR Apache-2.0 dual license.
