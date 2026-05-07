# JSONLogic Compatibility Matrix

`jsonlogic-fast` implements broad [JsonLogic](https://jsonlogic.com/) specification coverage by wrapping the mature [`jsonlogic-rs`](https://crates.io/crates/jsonlogic) crate.

## Supported Operators

All standard JSONLogic operators are supported:

| Category | Operators | Status | Notes |
|---|---|---|---|
| **Access** | `var`, `missing`, `missing_some` | ✅ Full | Deep paths (`a.b.c`) and arrays are supported. |
| **Logic** | `if`, `==`, `===`, `!=`, `!==`, `!`, `!!`, `or`, `and` | ✅ Full | `===` and `!==` map to strict equality matching JSONLogic spec. |
| **Numeric** | `>`, `>=`, `<`, `<=`, `max`, `min` | ✅ Full | Coerces strings to numbers following JS semantics. |
| **Arithmetic**| `+`, `-`, `*`, `/`, `%` | ✅ Full | Follows floating point logic. |
| **String** | `cat`, `substr`, `in` | ✅ Full | `in` checks substring inclusion. |
| **Array** | `map`, `filter`, `reduce`, `all`, `none`, `some`, `merge`, `in` | ✅ Full | `in` checks element presence. |

## Divergences & Edge Cases

The underlying `jsonlogic-rs` mostly mirrors the JavaScript reference implementation, but some edge cases might differ due to Rust's strong typing or different handling of numeric limits (e.g., `NaN` vs `null`).

- **Error Handling**: The standard `evaluate_batch` methods maintain backward-compatible, "lossy" behavior: if a context fails, they return `null` (or `0.0`) in that position to avoid failing the entire batch. The `_strict` variants (e.g., `evaluate_batch_strict`, `evaluate_batch_numeric_strict`) fail fast and return a top-level error if any context is invalid or fails to evaluate/coerce. Note that `_strict` variants prioritize fail-fast semantics over parallelism and currently execute sequentially.
- **Floating point accuracy**: Standard IEEE 754 precision applies. Extremely large integers or precise fractions might exhibit standard floating-point quirks.
- **Custom Operations**: Currently, adding custom operations is not exposed in the fast bindings layer, though it is technically possible in the underlying crate.

If you encounter a compatibility issue, please report it! We use `proptest` and official test suites to maintain adherence.
