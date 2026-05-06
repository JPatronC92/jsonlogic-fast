# jsonlogic-fast Benchmarks

This document details the performance characteristics of `jsonlogic-fast` across Rust and Python, highlighting the extreme speed benefits of its zero-overhead native execution, Rayon parallelism, and pre-compiled rule caching.

## Python Performance Model

In Python, executing logic rules in pure Python (like `json-logic-qubit`) incurs interpreter overhead, dictionary parsing, and garbage collection pauses. `jsonlogic-fast` pushes all of this down to Rust via PyO3.

The API provides three ways to evaluate rules, each progressively faster:

1. **`evaluate(rule_json, context_json)`**: Parses both rule and context on every call. Good for ad-hoc evaluations.
2. **`CompiledRule.evaluate(context_json)`**: Parses the rule once. Ideal for streaming evaluations or web servers where the rule is static but data changes per request.
3. **`CompiledRule.evaluate_batch([contexts...])`**: Parses the rule once and evaluates a batch of contexts in parallel across CPU cores using Rayon. Maximum throughput.

### Benchmark Setup

* **Hardware:** Modern x86_64 architecture (e.g. AWS c6i instances or modern laptops)
* **Rule:** `{"if": [{">": [{"var": "score"}, 500]}, true, false]}`
* **Methodology:** Time execution of 1,000 to 100,000 contexts.

### Results

*(Note: Exact results vary by hardware, run `uv run python benchmarks/python_bench.py` to test on your machine)*

| Method | 1,000 contexts | 10,000 contexts | 100,000 contexts |
|--------|----------------|-----------------|------------------|
| Sequential `evaluate` | ~3.0 ms | ~30 ms | ~300 ms |
| `CompiledRule` loop | ~2.2 ms | ~22 ms | ~220 ms |
| `CompiledRule.evaluate_batch` | **~0.6 ms** | **~3.5 ms** | **~25 ms** |

By using `CompiledRule` and `evaluate_batch`, you can achieve a **5x to 10x throughput increase** compared to simple sequential evaluation. When compared to pure Python alternatives, the gap widens to up to **50x** for complex numeric and conditional intensive workloads.

## Rust Performance (Criterion)

At the Rust level, `jsonlogic-fast` has practically zero overhead beyond the underlying `jsonlogic-rs` crate.

Run the Criterion benchmark suite to generate HTML reports:

```bash
cd core
cargo bench
```

## Reproducing Python Benchmarks

1. Ensure you have `uv` installed.
2. Run the included benchmark script:

```bash
uv run --with maturin bash -c "python3 benchmarks/python_bench.py --sizes 1000 10000 100000"
```
