use std::collections::HashMap;
use alloy_primitives::U256;

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
pub fn rate_limit_allow(now: u64, window_secs: u64, max_requests: u32, timestamps: &mut Vec<u64>) -> bool {
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