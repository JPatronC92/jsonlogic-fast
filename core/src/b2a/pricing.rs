use serde_json::Value;

pub fn estimate_cost(rule_depth: usize, batch_size: usize) -> f64 {
    let base_cost = 0.0001; // $0.0001 per base eval
    let depth_multiplier = 1.0 + (rule_depth as f64 * 0.1); // 10% more per depth level

    base_cost * depth_multiplier * (batch_size as f64)
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
}
