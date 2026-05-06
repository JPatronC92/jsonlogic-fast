use jsonlogic_fast::CompiledRule;
use serde_json::json;

fn main() {
    println!("--- Dynamic Feature Flag Evaluation ---");

    // Feature flag: "new_checkout_flow"
    // Rollout: 100% for internal users, 50% for beta users, 0% for everyone else
    let feature_rule = CompiledRule::new(r#"
        {"if": [
            {"==": [{"var": "user.is_internal"}, true]}, true,
            {"and": [
                {"==": [{"var": "user.is_beta"}, true]},
                {"<": [{"%": [{"var": "user.id"}, 100]}, 50]}
            ]}, true,
            false
        ]}
    "#).unwrap();

    let contexts = vec![
        json!({"user": {"id": 1, "is_internal": true, "is_beta": false}}),
        json!({"user": {"id": 25, "is_internal": false, "is_beta": true}}), // 25 % 100 < 50
        json!({"user": {"id": 75, "is_internal": false, "is_beta": true}}), // 75 % 100 >= 50
        json!({"user": {"id": 100, "is_internal": false, "is_beta": false}}),
    ];

    for ctx in contexts {
        let result = feature_rule.evaluate_value(&ctx).unwrap();
        println!("User ID {} gets new checkout: {}", ctx["user"]["id"], result);
    }
}
