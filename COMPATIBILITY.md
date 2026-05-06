# JSONLogic Compatibility Matrix

`jsonlogic-fast` implements the full [JsonLogic](https://jsonlogic.com/) specification by wrapping the mature [`jsonlogic-rs`](https://crates.io/crates/jsonlogic) crate.

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

- **Error Handling**: `jsonlogic-fast`'s batch evaluation methods will return `Null` / `0.0` for failures by default to avoid failing an entire batch due to one bad record. You can use the new `_strict` variants (e.g., `evaluate_batch_strict`) if you prefer an error to be raised instead.
- **Floating point accuracy**: Standard IEEE 754 precision applies. Extremely large integers or precise fractions might exhibit standard floating-point quirks.
- **Custom Operations**: Currently, adding custom operations is not exposed in the fast bindings layer, though it is technically possible in the underlying crate.

If you encounter a compatibility issue, please report it! We use `proptest` and official test suites to maintain adherence.
