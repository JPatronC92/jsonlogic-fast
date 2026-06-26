use alloy_primitives::U256;
use aws_sdk_dynamodb::types::AttributeValue;
use std::collections::HashMap;

/// Default balance for unseen addresses (no auto-seed of 10).
pub fn default_balance() -> U256 {
    U256::ZERO
}

/// Returns true if the nonce is allowed (new), false if replay (seen within ttl).
/// Cleans expired on check. `seen` is mutated.
pub fn nonce_allow(now: u64, ttl: u64, key: &str, seen: &mut HashMap<String, u64>) -> bool {
    seen.retain(|_, &mut ts| now - ts < ttl);
    if seen.contains_key(key) {
        false
    } else {
        seen.insert(key.to_string(), now);
        true
    }
}

/// Returns true if request allowed within window (len < max after clean).
/// `timestamps` is the list for this address, mutated (push if allow).
pub fn rate_limit_allow(
    now: u64,
    window_secs: u64,
    max_requests: u32,
    timestamps: &mut Vec<u64>,
) -> bool {
    timestamps.retain(|&ts| now - ts < window_secs);
    if timestamps.len() as u32 >= max_requests {
        false
    } else {
        timestamps.push(now);
        true
    }
}

#[cfg(test)]
mod hardening_logic_tests {
    use super::*;

    #[test]
    fn test_default_balance_zero() {
        assert_eq!(default_balance(), U256::ZERO);
    }

    #[test]
    fn test_nonce_replay() {
        let mut seen = HashMap::new();
        let now = 1_000_000u64;
        let ttl = 3600;
        let key = "0xaddr#n1";
        assert!(nonce_allow(now, ttl, key, &mut seen));
        assert!(!nonce_allow(now + 10, ttl, key, &mut seen)); // replay
        assert!(nonce_allow(now + 10, ttl, "0xaddr#n2", &mut seen));
    }

    #[test]
    fn test_rate_window() {
        let mut ts = vec![];
        let now = 1_000_000u64;
        let win = 60;
        let max = 10u32;
        for i in 0..10 {
            assert!(rate_limit_allow(now + i, win, max, &mut ts));
        }
        assert!(!rate_limit_allow(now + 10, win, max, &mut ts)); // 11th deny
                                                                 // after window, allows again
        assert!(rate_limit_allow(now + 100, win, max, &mut ts));
    }
}

#[cfg(test)]
mod dynamo_adapter_tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_dynamo_balance_miss_is_zero() {
        assert_eq!(balance_from_item(None), default_balance());
        let mut item = HashMap::new();
        item.insert("other".to_string(), AttributeValue::S("foo".into()));
        assert_eq!(balance_from_item(Some(&item)), default_balance());
    }

    #[test]
    fn test_dynamo_rate_10_allow_11_deny() {
        let mut ts = vec![];
        let now = 1_000_000u64;
        let win = 60;
        let max = 10u32;
        for i in 0..10 {
            assert!(dynamo_rate_decide(now + i, win, max, &mut ts));
        }
        assert!(!dynamo_rate_decide(now + 10, win, max, &mut ts));
    }

    #[test]
    fn test_dynamo_nonce_replay_maps_false() {
        assert_eq!(
            interpret_nonce_put("...ConditionalCheckFailedException..."),
            Ok(false)
        );
        assert!(interpret_nonce_put("other err").is_err());
        // success case (no err from put) is handled as Ok(true) in wrapper, not passed to interpret
    }

    #[test]
    fn test_dynamo_rate_ts_roundtrip() {
        let ts = vec![100, 200];
        let s = serialize_rate_ts(&ts);
        let back = parse_rate_ts(&s);
        assert_eq!(back, ts);
        assert!(parse_rate_ts("").is_empty());
    }

    #[test]
    fn test_rate_put_condition_first_write() {
        let (expr, vals) = rate_put_condition("");
        assert!(expr.contains("attribute_not_exists"));
        assert!(vals.is_none());
    }

    #[test]
    fn test_rate_put_condition_existing() {
        let (expr, vals) = rate_put_condition("100,200");
        assert!(expr.contains("rate_ts = :old"));
        assert!(vals.is_some());
    }

    #[test]
    fn test_include_user_balance_key_filters_meta_and_rate() {
        assert!(include_user_balance_key("0xabc"));
        assert!(!include_user_balance_key("__meta:last_sync_block"));
        assert!(!include_user_balance_key("__rate:0xfoo"));
    }
}

/// Adapter for Dynamo get_balance: returns default on miss.
pub(crate) fn balance_from_item(item: Option<&HashMap<String, AttributeValue>>) -> U256 {
    if let Some(it) = item {
        if let Some(AttributeValue::S(bal_str)) = it.get("balance") {
            return bal_str.parse::<U256>().unwrap_or(default_balance());
        }
        // legacy N
        if let Some(AttributeValue::N(bal_str)) = it.get("balance") {
            if let Ok(f) = bal_str.parse::<f64>() {
                return U256::from((f * 1e18) as u128);
            }
        }
    }
    default_balance()
}

/// Parse "ts1,ts2,..." to Vec<u64> for rate.
pub(crate) fn parse_rate_ts(s: &str) -> Vec<u64> {
    if s.is_empty() {
        return vec![];
    }
    s.split(',').filter_map(|p| p.parse::<u64>().ok()).collect()
}

/// Serialize Vec to "ts1,ts2,..."
pub(crate) fn serialize_rate_ts(ts: &[u64]) -> String {
    ts.iter()
        .map(|t| t.to_string())
        .collect::<Vec<_>>()
        .join(",")
}

/// Decision for Dynamo rate: load vec, call pure allow, return bool.
pub(crate) fn dynamo_rate_decide(
    now: u64,
    window_secs: u64,
    max_requests: u32,
    timestamps: &mut Vec<u64>,
) -> bool {
    rate_limit_allow(now, window_secs, max_requests, timestamps)
}

/// Interpret result of nonce put for Dynamo: Conditional fail -> false (replay), ok -> true, else err.
pub(crate) fn interpret_nonce_put(err_debug: &str) -> Result<bool, String> {
    if err_debug.contains("ConditionalCheckFailedException") {
        Ok(false)
    } else {
        Err(format!("DynamoDB nonce error: {}", err_debug))
    }
}

/// Pure helper extracted for Dynamo rate conditional put (used by storage to build ConditionExpression + values).
pub(crate) fn rate_put_condition(
    read_serialized: &str,
) -> (String, Option<(String, AttributeValue)>) {
    if read_serialized.is_empty() {
        ("attribute_not_exists(rate_ts)".to_string(), None)
    } else {
        (
            "rate_ts = :old".to_string(),
            Some((
                ":old".to_string(),
                AttributeValue::S(read_serialized.to_string()),
            )),
        )
    }
}

/// Pure filter: include only user balance keys (exclude __meta:* and __rate:* ).
pub(crate) fn include_user_balance_key(addr: &str) -> bool {
    !addr.starts_with("__")
}

/// Condition expression for Dynamo add_balance. Allows first deposit when the item does not exist,
/// while preserving optimistic concurrency when a balance already exists.
pub(crate) fn dynamo_add_balance_condition() -> &'static str {
    "attribute_not_exists(balance) OR balance = :current_balance"
}

#[cfg(test)]
mod dynamo_add_balance_tests {
    use super::*;

    #[test]
    fn test_dynamo_add_balance_allows_new_items() {
        let expr = dynamo_add_balance_condition();
        assert!(expr.contains("attribute_not_exists(balance)"));
        assert!(expr.contains("balance = :current_balance"));
    }
}
