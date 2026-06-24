use alloy_primitives::U256;
use serde_json::Value;

/// Base cost: 0.0001 with 18 decimals (same unit as on-chain wei deposits)
const BASE_COST: u128 = 100_000_000_000_000;           // 0.0001 * 10^18
const DEPTH_BONUS_PER_LEVEL: u128 = 10_000_000_000_000; // 0.00001 * 10^18 (10% of base)

/// Returns the cost of an evaluation as U256 (18 decimal places precision).
/// This eliminates all floating point conversion issues with large integers.
pub fn estimate_cost(rule_depth: usize, batch_size: usize) -> U256 {
    let base = U256::from(BASE_COST);
    let bonus = U256::from(DEPTH_BONUS_PER_LEVEL) * U256::from(rule_depth as u64);
    let cost_per_eval = base + bonus;
    cost_per_eval * U256::from(batch_size as u64)
}

/// Dynamically calculates the logical depth of a JSONLogic rule tree.
pub fn calculate_rule_depth(rule: &Value) -> usize {
    match rule {
        Value::Object(obj) => {
            let max_val_depth = obj.values().map(calculate_rule_depth).max().unwrap_or(0);
            1 + max_val_depth
        }
        Value::Array(arr) => arr.iter().map(calculate_rule_depth).max().unwrap_or(0),
        _ => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_calculate_rule_depth() {
        assert_eq!(calculate_rule_depth(&json!(1)), 0);
        assert_eq!(calculate_rule_depth(&json!("hello")), 0);
        assert_eq!(calculate_rule_depth(&json!({"var": "x"})), 1);
        assert_eq!(calculate_rule_depth(&json!({"==": [{"var": "x"}, 5]})), 2);
        assert_eq!(
            calculate_rule_depth(&json!({
                "and": [
                    {"==": [{"var": "temp"}, 100]},
                    {"==": [{"var": "status"}, "active"]}
                ]
            })),
            3
        );
    }

    #[test]
    fn test_estimate_cost_u256() {
        // depth 3, batch 10:
        // old: 0.0001 * 1.3 * 10 = 0.0013
        // new: 1.3e15  (0.0013 * 10^18)
        let cost = estimate_cost(3, 10);
        let expected: U256 = U256::from(1_300_000_000_000_000u128);
        assert_eq!(cost, expected);

        // depth 2, batch 1: 0.00012 * 1e18 = 120_000_000_000_000
        let cost2 = estimate_cost(2, 1);
        let expected2: U256 = U256::from(120_000_000_000_000u128);
        assert_eq!(cost2, expected2);
    }
}
