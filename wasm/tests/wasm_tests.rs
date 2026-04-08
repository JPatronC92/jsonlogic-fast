use wasm_bindgen_test::*;

use jsonlogic_fast_wasm::*;

#[wasm_bindgen_test]
fn evaluate_string_result() {
    let rule = r#"{"if":[{">":[{"var":"score"},700]},"approve","review"]}"#;
    let ctx = r#"{"score":742}"#;
    let result = evaluate_wasm(rule, ctx).unwrap();
    assert_eq!(result, "\"approve\"");
}

#[wasm_bindgen_test]
fn evaluate_boolean_result() {
    let rule = r#"{">":[{"var":"score"},700]}"#;
    let ctx = r#"{"score":800}"#;
    let result = evaluate_wasm(rule, ctx).unwrap();
    assert_eq!(result, "true");
}

#[wasm_bindgen_test]
fn evaluate_invalid_rule_returns_error() {
    let result = evaluate_wasm("not json", r#"{"x":1}"#);
    assert!(result.is_err());
}

#[wasm_bindgen_test]
fn evaluate_invalid_context_returns_error() {
    let rule = r#"{">":[{"var":"x"},1]}"#;
    let result = evaluate_wasm(rule, "bad json");
    assert!(result.is_err());
}

#[wasm_bindgen_test]
fn evaluate_numeric_basic() {
    let rule = r#"{"*":[{"var":"amount"},0.029]}"#;
    let ctx = r#"{"amount":1000}"#;
    let result = evaluate_numeric_wasm(rule, ctx).unwrap();
    assert!((result - 29.0).abs() < 1e-6);
}

#[wasm_bindgen_test]
fn evaluate_numeric_boolean_coercion() {
    let rule = r#"{"==":[1,1]}"#;
    let ctx = r#"{}"#;
    let result = evaluate_numeric_wasm(rule, ctx).unwrap();
    assert!((result - 1.0).abs() < 1e-6);
}

#[wasm_bindgen_test]
fn evaluate_batch_multiple_contexts() {
    let rule = r#"{"if":[{">":[{"var":"score"},700]},"approve","review"]}"#;
    let contexts = r#"[{"score":800},{"score":500},{"score":742}]"#;
    let result = evaluate_batch_wasm(rule, contexts).unwrap();
    let parsed: Vec<String> = serde_json::from_str(&result).unwrap();
    assert_eq!(parsed, vec!["approve", "review", "approve"]);
}

#[wasm_bindgen_test]
fn evaluate_batch_empty_array() {
    let rule = r#"{"==":[1,1]}"#;
    let contexts = r#"[]"#;
    let result = evaluate_batch_wasm(rule, contexts).unwrap();
    let parsed: Vec<serde_json::Value> = serde_json::from_str(&result).unwrap();
    assert!(parsed.is_empty());
}

#[wasm_bindgen_test]
fn evaluate_batch_detailed_success() {
    let rule = r#"{">":[{"var":"x"},1]}"#;
    let contexts = r#"[{"x":5}]"#;
    let result = evaluate_batch_detailed_wasm(rule, contexts).unwrap();
    let parsed: Vec<serde_json::Value> = serde_json::from_str(&result).unwrap();
    assert_eq!(parsed.len(), 1);
    assert_eq!(parsed[0]["result"], true);
    assert!(parsed[0]["error"].is_null());
}

#[wasm_bindgen_test]
fn evaluate_batch_numeric_results() {
    let rule = r#"{"*":[{"var":"amount"},0.029]}"#;
    let contexts = r#"[{"amount":100},{"amount":200}]"#;
    let result = evaluate_batch_numeric_wasm(rule, contexts).unwrap();
    let parsed: Vec<f64> = serde_json::from_str(&result).unwrap();
    assert_eq!(parsed.len(), 2);
    assert!((parsed[0] - 2.9).abs() < 1e-6);
    assert!((parsed[1] - 5.8).abs() < 1e-6);
}

#[wasm_bindgen_test]
fn validate_valid_rule() {
    let result = validate_rule_wasm(r#"{">":[{"var":"x"},1]}"#).unwrap();
    assert!(result);
}

#[wasm_bindgen_test]
fn validate_invalid_json_returns_error() {
    let result = validate_rule_wasm("not json");
    assert!(result.is_err());
}

#[wasm_bindgen_test]
fn core_info_returns_valid_json() {
    let info = get_core_info_wasm();
    let parsed: serde_json::Value = serde_json::from_str(&info).unwrap();
    assert!(parsed.get("version").is_some());
}

#[wasm_bindgen_test]
fn determinism_repeated_evaluation() {
    let rule = r#"{"if":[{">":[{"var":"score"},700]},"approve","review"]}"#;
    let ctx = r#"{"score":742,"country":"MX"}"#;
    let first = evaluate_wasm(rule, ctx).unwrap();
    for _ in 0..50 {
        assert_eq!(evaluate_wasm(rule, ctx).unwrap(), first);
    }
}

#[wasm_bindgen_test]
fn determinism_batch_matches_individual() {
    let rule = r#"{"if":[{">":[{"var":"score"},700]},"approve","review"]}"#;
    let contexts_json = r#"[{"score":800},{"score":500}]"#;
    let batch_result = evaluate_batch_wasm(rule, contexts_json).unwrap();
    let batch: Vec<String> = serde_json::from_str(&batch_result).unwrap();

    let individual_1 = evaluate_wasm(rule, r#"{"score":800}"#).unwrap();
    let individual_2 = evaluate_wasm(rule, r#"{"score":500}"#).unwrap();

    assert_eq!(format!("\"{}\"", batch[0]), individual_1);
    assert_eq!(format!("\"{}\"", batch[1]), individual_2);
}

#[wasm_bindgen_test]
fn evaluate_batch_detailed_mixed_results() {
    let rule = r#"{"var":"x"}"#;
    // Note: The WASM wrapper parses contexts_json as a whole first,
    // so individual contexts are always valid JSON when they reach the core.
    let contexts = r#"[{"x":10},{},{"x":null}]"#;
    let result = evaluate_batch_detailed_wasm(rule, contexts).unwrap();
    let parsed: Vec<serde_json::Value> = serde_json::from_str(&result).unwrap();

    assert_eq!(parsed.len(), 3);

    // Context 1: Success
    assert_eq!(parsed[0]["result"], 10.0);
    assert!(parsed[0]["error"].is_null());

    // Context 2: Missing variable (returns null)
    assert!(parsed[1]["result"].is_null());
    assert!(parsed[1]["error"].is_null());

    // Context 3: Explicit null
    assert!(parsed[2]["result"].is_null());
    assert!(parsed[2]["error"].is_null());
}

#[wasm_bindgen_test]
fn evaluate_batch_numeric_detailed_mixed_results() {
    let rule = r#"{"if":[{"==":[{"var":"type"},"good"]},10,"bad_result"]}"#;
    let contexts = r#"[{"type":"good"},{"type":"bad"}]"#;
    let result = evaluate_batch_numeric_detailed_wasm(rule, contexts).unwrap();
    let parsed: Vec<serde_json::Value> = serde_json::from_str(&result).unwrap();

    assert_eq!(parsed.len(), 2);

    // Context 1: Success
    assert_eq!(parsed[0]["result"], 10.0);
    assert!(parsed[0]["error"].is_null());

    // Context 2: Coercion error
    assert_eq!(parsed[1]["result"], 0.0);
    assert!(parsed[1]["error"].is_string());
    assert!(parsed[1]["error"]
        .as_str()
        .unwrap()
        .contains("Numeric coercion error"));
}

#[wasm_bindgen_test]
fn evaluate_batch_detailed_invalid_contexts_json() {
    let rule = r#"{"var":"x"}"#;
    let contexts = r#"[{"x":10}, invalid]"#;
    let result = evaluate_batch_detailed_wasm(rule, contexts);

    assert!(result.is_err());
}
