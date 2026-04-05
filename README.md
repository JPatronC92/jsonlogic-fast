# jsonlogic-fast

[![crates.io](https://img.shields.io/crates/v/jsonlogic-fast.svg)](https://crates.io/crates/jsonlogic-fast)
[![docs.rs](https://docs.rs/jsonlogic-fast/badge.svg)](https://docs.rs/jsonlogic-fast)
[![CI](https://github.com/JPatronC92/jsonlogic-fast/actions/workflows/ci.yml/badge.svg)](https://github.com/JPatronC92/jsonlogic-fast/actions/workflows/ci.yml)

**Fast, embeddable, cross-runtime JSON-Logic evaluation.**

## Why

- **Rust core** — zero-overhead evaluation with strong type safety
- **Python bindings** — native speed via PyO3, no subprocess overhead
- **WASM bindings** — run in browsers and Node.js, same deterministic results
- **Parallel batch** — Rayon-powered multi-threaded evaluation on native platforms
- **Deterministic** — same rule + same data = same result, always

## Install

### Rust

```toml
[dependencies]
jsonlogic-fast = "0.1"
```

### Python

```bash
pip install jsonlogic-fast
```

## Quick Start

### Rust

```rust
use jsonlogic_fast::evaluate;

let rule = r#"{"if":[{">":[{"var":"score"},700]},"approve","review"]}"#;
let context = r#"{"score":742}"#;
let result = evaluate(rule, context).unwrap();
// result == json!("approve")
```

### Python

```python
import json
import jsonlogic_fast

rule = json.dumps({">": [{"var": "score"}, 700]})
ctx = json.dumps({"score": 742})

result = jsonlogic_fast.evaluate(rule, ctx)  # True
```

### JavaScript (WASM)

```javascript
import { evaluate_wasm } from "jsonlogic-fast-wasm";

const result = evaluate_wasm(
  '{">":[{"var":"score"},700]}',
  '{"score":742}'
);
// result == "true"
```

## API

| Function | Return | Description |
|---|---|---|
| `evaluate` | `Value` | Evaluate rule against context |
| `evaluate_numeric` | `f64` | Evaluate and coerce to number |
| `evaluate_batch` | `Vec<Value>` | Parallel evaluation of N contexts |
| `evaluate_batch_detailed` | `Vec<EvaluationResult>` | Batch with per-item error info |
| `evaluate_batch_numeric` | `Vec<f64>` | Batch numeric with fail-safe zeros |
| `evaluate_batch_numeric_detailed` | `Vec<NumericEvaluationResult>` | Batch numeric with errors |
| `validate_rule` | `bool` | Preflight rule validation |
| `serialize` | `String` | JSON serialization |
| `get_core_info` | `Value` | Engine metadata |

## Benchmarks

```bash
make bench          # full Criterion benchmarks
make bench-quick    # reduced sample for quick feedback
```

Included benchmarks:
- Single numeric evaluation
- Single generic (conditional) evaluation
- Batch numeric evaluation (10K contexts)

## Performance

`jsonlogic-fast` wraps the [jsonlogic-rs](https://crates.io/crates/jsonlogic) crate
in a Rust core compiled to native code, exposed to Python via PyO3 and to browsers via
WASM. The "fast" comes from:

- **Zero Python overhead** — rule parsing, evaluation, and serialization happen entirely in Rust.
- **Parallel batch evaluation** — Rayon distributes N contexts across all available CPU cores.
- **Single parse, many evaluations** — `evaluate_batch` parses the rule once and reuses it.

### Python: jsonlogic-fast vs json-logic-qubit (pure Python)

| Scenario | jsonlogic-fast | json-logic-qubit | Speedup |
|---|---|---|---|
| Simple comparison | 0.001 ms | 0.004 ms | **3.6x** |
| Conditional (nested if) | 0.003 ms | 0.011 ms | **4.0x** |
| Complex (logic + arithmetic + var paths) | 0.005 ms | 0.025 ms | **4.7x** |
| Batch 10K contexts | 5.3 ms | 51.4 ms (sequential loop) | **9.7x** |

> Run `make bench-python` to reproduce. Run `make bench` for Criterion (Rust) benchmarks.

### Rust (Criterion)

```bash
make bench          # full Criterion benchmarks (HTML reports in target/criterion/)
make bench-quick    # reduced sample for quick feedback
```

## Development

```bash
make test           # core Rust tests (11)
make test-python    # Python e2e tests (29)
make test-wasm      # WASM runtime tests (15)
make ci-local       # full quality gate
```

## License

Licensed under either of

- [MIT license](LICENSE-MIT)
- [Apache License, Version 2.0](LICENSE-APACHE)

at your option.
