use jsonlogic;
use serde_json::Value;
use wasm_bindgen::prelude::*;

// ─────────────────────────────────────────────────────────────
// Helper: Extract numeric result from json-logic evaluation
// ─────────────────────────────────────────────────────────────
fn extract_numeric(result: Value) -> Result<f64, String> {
    match result {
        Value::Number(n) => n
            .as_f64()
            .ok_or_else(|| "Numeric result could not be converted to f64".to_string()),
        Value::String(s) => s
            .parse::<f64>()
            .map_err(|_| format!("String '{}' is not a valid number", s)),
        Value::Bool(b) => Ok(if b { 1.0 } else { 0.0 }),
        Value::Null => Ok(0.0),
        other => Err(format!(
            "Expected numeric, got: {}",
            serde_json::to_string(&other).unwrap_or_default()
        )),
    }
}

// ─────────────────────────────────────────────────────────────
// 1. Single fee evaluation (browser)
// ─────────────────────────────────────────────────────────────
#[wasm_bindgen]
pub fn evaluate_fee_wasm(rule_json: &str, context_json: &str) -> Result<f64, JsValue> {
    let rule: Value = serde_json::from_str(rule_json)
        .map_err(|e| JsValue::from_str(&format!("Rule parse error: {}", e)))?;

    let context: Value = serde_json::from_str(context_json)
        .map_err(|e| JsValue::from_str(&format!("Context parse error: {}", e)))?;

    let result = jsonlogic::apply(&rule, &context)
        .map_err(|e| JsValue::from_str(&format!("json-logic error: {}", e)))?;

    extract_numeric(result).map_err(|e| JsValue::from_str(&e))
}

// ─────────────────────────────────────────────────────────────
// 2. Batch evaluation (browser) — processes a JSON array of contexts
// ─────────────────────────────────────────────────────────────
#[wasm_bindgen]
pub fn evaluate_batch_wasm(rule_json: &str, contexts_json: &str) -> Result<String, JsValue> {
    let rule: Value = serde_json::from_str(rule_json)
        .map_err(|e| JsValue::from_str(&format!("Rule parse error: {}", e)))?;

    let contexts: Vec<Value> = serde_json::from_str(contexts_json)
        .map_err(|e| JsValue::from_str(&format!("Contexts parse error: {}", e)))?;

    let results: Vec<f64> = contexts
        .iter()
        .map(|ctx| match jsonlogic::apply(&rule, ctx) {
            Ok(val) => extract_numeric(val).unwrap_or(0.0),
            Err(_) => 0.0,
        })
        .collect();

    serde_json::to_string(&results)
        .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
}

// ─────────────────────────────────────────────────────────────
// 3. Multi-rule batch evaluation (browser) — evaluates array of rules
// ─────────────────────────────────────────────────────────────
#[wasm_bindgen]
pub fn evaluate_batch_multi_wasm(rules_json: &str, contexts_json: &str) -> Result<String, JsValue> {
    let rules: Vec<Value> = serde_json::from_str(rules_json)
        .map_err(|e| JsValue::from_str(&format!("Rules parse error: {}", e)))?;

    let contexts: Vec<Value> = serde_json::from_str(contexts_json)
        .map_err(|e| JsValue::from_str(&format!("Contexts parse error: {}", e)))?;

    let mut results: Vec<Value> = Vec::with_capacity(contexts.len());

    for ctx in &contexts {
        let mut rule_fees: Vec<f64> = Vec::with_capacity(rules.len());
        let mut total_fee = 0.0;

        for rule in &rules {
            let fee = match jsonlogic::apply(rule, ctx) {
                Ok(val) => extract_numeric(val).unwrap_or(0.0),
                Err(_) => 0.0,
            };
            rule_fees.push(fee);
            total_fee += fee;
        }

        results.push(serde_json::json!({
            "total_fee": total_fee,
            "rule_fees": rule_fees
        }));
    }

    serde_json::to_string(&results)
        .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
}

// ─────────────────────────────────────────────────────────────
// 3. Validate a json-logic rule
// ─────────────────────────────────────────────────────────────
#[wasm_bindgen]
pub fn validate_rule_wasm(rule_json: &str) -> Result<bool, JsValue> {
    let rule: Value = serde_json::from_str(rule_json)
        .map_err(|e| JsValue::from_str(&format!("Invalid JSON: {}", e)))?;

    let test_ctx = serde_json::json!({
        "amount": 100.0,
        "total_volume": 1000.0,
        "method": "CREDIT_CARD"
    });

    match jsonlogic::apply(&rule, &test_ctx) {
        Ok(_) => Ok(true),
        Err(e) => Err(JsValue::from_str(&format!("Validation failed: {}", e))),
    }
}

// ─────────────────────────────────────────────────────────────
// 4. Engine info (browser)
// ─────────────────────────────────────────────────────────────
#[wasm_bindgen]
pub fn get_core_info_wasm() -> String {
    serde_json::json!({
        "engine": "tempus_core",
        "version": env!("CARGO_PKG_VERSION"),
        "target": "wasm32",
        "runtime": "browser",
        "evaluator": "jsonlogic-rs",
    })
    .to_string()
}
