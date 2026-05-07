use jsonlogic_fast::CompiledRule;
use serde_json::json;

fn main() {
    println!("--- Real-time Fraud Scoring Engine ---");

    // A compiled rule that parses once and executes fast.
    let fraud_rule = CompiledRule::new(r#"
        {"if": [
            {">": [{"var": "amount"}, 10000]}, "block",
            {">": [{"var": "failed_attempts"}, 3]}, "block",
            {"in": [{"var": "country"}, ["US", "CA", "GB"]]}, "allow",
            "review"
        ]}
    "#).expect("Failed to compile rule");

    let contexts = vec![
        json!({"amount": 150, "failed_attempts": 0, "country": "US", "user_id": "u123"}),
        json!({"amount": 15000, "failed_attempts": 1, "country": "US", "user_id": "u124"}),
        json!({"amount": 50, "failed_attempts": 5, "country": "US", "user_id": "u125"}),
        json!({"amount": 200, "failed_attempts": 0, "country": "RU", "user_id": "u126"}),
    ];

    for context in contexts {
        let result = fraud_rule.evaluate_value(&context).unwrap();
        println!("User {} -> Decision: {}", context["user_id"], result);
    }
}
