use jsonlogic_fast::{evaluate, CompiledRule};
use proptest::prelude::*;
use serde_json::json;

proptest! {
    #[test]
    fn test_valid_json_never_panics(rule in proptest::string::string_regex(".*").unwrap(), context in proptest::string::string_regex(".*").unwrap()) {
        // Just verify it doesn't panic on random strings
        let _ = evaluate(&rule, &context);
    }

    #[test]
    fn test_arbitrary_nested_json(
        rule_json in r#"\{"(==|!=|>|<|>=|<=|\+|-|\*|/)":\s*\[\s*-?[0-9]+(\.[0-9]+)?\s*,\s*-?[0-9]+(\.[0-9]+)?\s*\]\}"#
    ) {
        let compiled = CompiledRule::new(&rule_json);
        if let Ok(rule) = compiled {
            let _ = rule.evaluate("{}");
        }
    }
}

// Proptest for arithmetic bounds and strange types
proptest! {
    #[test]
    fn test_addition_does_not_panic_on_weird_types(
        a in any::<f64>(),
        b in any::<f64>()
    ) {
        // JSONLogic handles NaNs and Infs differently sometimes, but it shouldn't panic
        let rule = json!({"+": [a, b]});
        let rule_str = serde_json::to_string(&rule).unwrap();
        let _ = evaluate(&rule_str, "{}");
    }
}
