#!/usr/bin/env python3
"""
Benchmark: jsonlogic-fast (Rust/PyO3) vs json-logic-py (pure Python).

Usage:
    make bench-python

Or manually:
    cd python && uv run --with maturin bash -c "maturin develop --release"
    uv run --with json-logic python benchmarks/compare.py
"""

import json
import timeit
from statistics import mean, stdev

# ---------------------------------------------------------------------------
# Engines
# ---------------------------------------------------------------------------

import jsonlogic_fast

try:
    from json_logic import jsonLogic as json_logic_py_eval

    HAS_JSON_LOGIC_PY = True
except ImportError:
    try:
        from json_logic_qubit import jsonLogic as json_logic_py_eval

        HAS_JSON_LOGIC_PY = True
    except ImportError:
        HAS_JSON_LOGIC_PY = False


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

ITERATIONS = 5_000
BATCH_SIZE = 10_000
REPEATS = 5


def _bench(label: str, fn, iterations: int = ITERATIONS, repeats: int = REPEATS):
    """Run *fn* in a timeit loop and return (mean_ms, stdev_ms)."""
    times = timeit.repeat(fn, number=iterations, repeat=repeats)
    per_call = [t / iterations * 1_000 for t in times]
    return mean(per_call), stdev(per_call)


def _fmt(ms: float, sd: float) -> str:
    if ms < 0.001:
        return f"{ms * 1_000:.2f} µs ± {sd * 1_000:.2f}"
    return f"{ms:.4f} ms ± {sd:.4f}"


# ---------------------------------------------------------------------------
# Scenarios
# ---------------------------------------------------------------------------

# 1. Simple comparison
SIMPLE_RULE = {">": [{"var": "score"}, 700]}
SIMPLE_DATA = {"score": 742}

# 2. Conditional with nested if
CONDITIONAL_RULE = {
    "if": [
        {">": [{"var": "score"}, 700]},
        "approve",
        {"if": [{">": [{"var": "score"}, 400]}, "review", "reject"]},
    ]
}
CONDITIONAL_DATA = {"score": 500}

# 3. Complex rule — arithmetic + logic + var paths
COMPLEX_RULE = {
    "if": [
        {
            "and": [
                {">": [{"var": "score"}, 700]},
                {"==": [{"var": "country"}, "MX"]},
                {"in": [{"var": "user.tier"}, ["gold", "platinum"]]},
            ]
        },
        {"*": [{"var": "amount"}, 0.025]},
        {"*": [{"var": "amount"}, 0.035]},
    ]
}
COMPLEX_DATA = {
    "score": 750,
    "country": "MX",
    "user": {"tier": "gold"},
    "amount": 1000,
}

# Pre-serialized for jsonlogic-fast (expects JSON strings)
SIMPLE_RULE_JSON = json.dumps(SIMPLE_RULE)
SIMPLE_DATA_JSON = json.dumps(SIMPLE_DATA)
CONDITIONAL_RULE_JSON = json.dumps(CONDITIONAL_RULE)
CONDITIONAL_DATA_JSON = json.dumps(CONDITIONAL_DATA)
COMPLEX_RULE_JSON = json.dumps(COMPLEX_RULE)
COMPLEX_DATA_JSON = json.dumps(COMPLEX_DATA)


# ---------------------------------------------------------------------------
# Batch scenario (jsonlogic-fast only — json-logic-py has no batch API)
# ---------------------------------------------------------------------------

BATCH_RULE_JSON = json.dumps({"*": [{"var": "amount"}, 0.029]})
BATCH_CONTEXTS = [json.dumps({"amount": float(i)}) for i in range(BATCH_SIZE)]


# ---------------------------------------------------------------------------
# Runner
# ---------------------------------------------------------------------------


def run_scenario(name: str, rule, data, rule_json: str, data_json: str):
    """Benchmark a single scenario for both engines."""
    # jsonlogic-fast
    fast_ms, fast_sd = _bench(
        f"fast/{name}", lambda: jsonlogic_fast.evaluate(rule_json, data_json)
    )

    if HAS_JSON_LOGIC_PY:
        py_ms, py_sd = _bench(
            f"py/{name}", lambda: json_logic_py_eval(rule, data)
        )
        speedup = py_ms / fast_ms if fast_ms > 0 else float("inf")
    else:
        py_ms, py_sd, speedup = None, None, None

    return fast_ms, fast_sd, py_ms, py_sd, speedup


def main() -> int:
    print("=" * 72)
    print("jsonlogic-fast (Rust/PyO3)  vs  json-logic-py (pure Python)")
    print(f"Iterations per scenario: {ITERATIONS:,}  |  Repeats: {REPEATS}")
    print("=" * 72)

    if not HAS_JSON_LOGIC_PY:
        print(
            "\n⚠  json-logic (json-logic-py) not installed."
            "\n   Only jsonlogic-fast numbers will be shown."
            "\n   Install with: pip install json-logic\n"
        )

    scenarios = [
        ("simple_comparison", SIMPLE_RULE, SIMPLE_DATA, SIMPLE_RULE_JSON, SIMPLE_DATA_JSON),
        ("conditional_nested_if", CONDITIONAL_RULE, CONDITIONAL_DATA, CONDITIONAL_RULE_JSON, CONDITIONAL_DATA_JSON),
        ("complex_multi_op", COMPLEX_RULE, COMPLEX_DATA, COMPLEX_RULE_JSON, COMPLEX_DATA_JSON),
    ]

    # Header
    if HAS_JSON_LOGIC_PY:
        print(
            f"\n{'Scenario':<25} {'jsonlogic-fast':>22} {'json-logic-py':>22} {'Speedup':>10}"
        )
        print("-" * 82)
    else:
        print(f"\n{'Scenario':<25} {'jsonlogic-fast':>22}")
        print("-" * 50)

    for name, rule, data, rule_json, data_json in scenarios:
        fast_ms, fast_sd, py_ms, py_sd, speedup = run_scenario(
            name, rule, data, rule_json, data_json
        )
        fast_str = _fmt(fast_ms, fast_sd)
        if HAS_JSON_LOGIC_PY and py_ms is not None:
            py_str = _fmt(py_ms, py_sd)
            print(f"{name:<25} {fast_str:>22} {py_str:>22} {speedup:>9.1f}x")
        else:
            print(f"{name:<25} {fast_str:>22}")

    # Batch (jsonlogic-fast only)
    print(f"\n{'Batch evaluation':^72}")
    print("-" * 72)
    batch_ms, batch_sd = _bench(
        "batch_10k",
        lambda: jsonlogic_fast.evaluate_batch(BATCH_RULE_JSON, BATCH_CONTEXTS),
        iterations=50,
        repeats=REPEATS,
    )
    print(f"{'batch_numeric_10k':<25} {_fmt(batch_ms, batch_sd):>22}  ({BATCH_SIZE:,} contexts)")

    if HAS_JSON_LOGIC_PY:
        batch_py_ms, batch_py_sd = _bench(
            "batch_10k_py",
            lambda: [json_logic_py_eval({"*": [{"var": "amount"}, 0.029]}, {"amount": float(i)}) for i in range(BATCH_SIZE)],
            iterations=5,
            repeats=REPEATS,
        )
        speedup = batch_py_ms / batch_ms if batch_ms > 0 else float("inf")
        print(f"{'batch_10k (py loop)':<25} {_fmt(batch_py_ms, batch_py_sd):>22}  (sequential loop, {speedup:.1f}x slower)")

    print("\n" + "=" * 72)
    print("Done.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
