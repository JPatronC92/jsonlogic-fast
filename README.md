# jsonlogic-fast

[![crates.io](https://img.shields.io/crates/v/jsonlogic-fast.svg)](https://crates.io/crates/jsonlogic-fast)
[![docs.rs](https://docs.rs/jsonlogic-fast/badge.svg)](https://docs.rs/jsonlogic-fast)
[![CI](https://github.com/JPatronC92/jsonlogic-fast/actions/workflows/ci.yml/badge.svg)](https://github.com/JPatronC92/jsonlogic-fast/actions/workflows/ci.yml)

**Deterministic JSON-Logic evaluation designed for repeated evaluation across Rust, Python, and WASM.**

`jsonlogic-fast` is a production-oriented wrapper around the highly-compliant [`jsonlogic-rs`](https://crates.io/crates/jsonlogic) crate, offering bindings that are significantly faster than native Python or JS equivalents.

## When to use this

* **Fraud / Risk Scoring Engine:** You need to evaluate high-throughput evaluations against dynamic JSON rules.
* **Feature Flagging:** You have complex targeting rules that need to be evaluated synchronously without network latency.
* **Multi-Platform Consistency:** You need the exact same rule to evaluate identically on your Rust backend, your Python data pipeline, and your user's browser (WASM).
* **AI Guardrails:** You want deterministic, programmatic checks on LLM JSON outputs before accepting them.

## Why not just use pure Python?

Pure Python implementations (like `json-logic-qubit`) are fine for simple scripts, but they suffer from interpreter overhead, GC pauses, and poor parallelism. `jsonlogic-fast` drops down to native Rust, parsing and executing your rules with zero Python overhead. Using our `CompiledRule.evaluate_batch` API, you can parallelize evaluation across all CPU cores instantly, achieving **up to significant speedups (benchmark results depend on rule shape, batch size, hardware, and runtime)**.

See [BENCHMARKS.md](BENCHMARKS.md) for detailed performance stats.

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

### WASM

```bash
npm install jsonlogic-fast-wasm
```

## Quick Start

### Python: The "Compiled" API for Production

Parse once, evaluate many times natively.

```python
import json
from jsonlogic_fast import CompiledRule

rule = CompiledRule('{"if": [{">": [{"var": "score"}, 700]}, "approve", "review"]}')

# Single eval
print(rule.evaluate('{"score": 742}')) # 'approve'

# Parallel batch eval (utilizes all CPU cores automatically)
contexts = ['{"score": 742}', '{"score": 600}']
print(rule.evaluate_batch(contexts)) # ['approve', 'review']
```

### Rust

```rust
use jsonlogic_fast::CompiledRule;
use serde_json::json;

let rule = CompiledRule::new(r#"{"if":[{">":[{"var":"score"},700]},"approve","review"]}"#).unwrap();
let result = rule.evaluate_value(&json!({"score": 742})).unwrap();
assert_eq!(result, json!("approve"));
```

### JavaScript (WASM)

```javascript
import init, { CompiledRule } from "jsonlogic-fast-wasm";

await init();

const rule = new CompiledRule('{"if": [{">": [{"var": "score"}, 700]}, "approve", "review"]}');
const result = rule.evaluate('{"score": 742}');
console.log(result); // "approve"
```

## API Features

| Feature | Description |
|---|---|
| `CompiledRule` | Parses a rule string once into a native memory representation for fast repeated evaluations. |
| `evaluate_batch` | Evaluates an array of contexts against a rule in parallel (Rayon on native). Maintains backward compatibility and is "lossy" (returns `null` in the array position if a context is invalid or fails). |
| `evaluate_batch_strict` | Batch evaluation that prioritizes fail-fast semantics over parallelism. It returns a top-level Error immediately if any context is invalid or fails evaluation. Currently runs sequentially. |
| `evaluate_batch_numeric_strict` | Strict batch evaluation that also fails if a result cannot be coerced to a number. |
| `evaluate_numeric` | Evaluates and coerces the result safely to a Float (`f64`), useful for scoring. |

## Compatibility & Limitations

We implement the broad JSON-Logic operator coverage. See [COMPATIBILITY.md](COMPATIBILITY.md) for an operator support matrix and edge-case behavior (e.g. floats and strict equality).

## License

MIT or Apache 2.0.
