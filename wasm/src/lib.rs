use jsonlogic_fast::{
    evaluate as core_evaluate, evaluate_batch as core_evaluate_batch,
    evaluate_batch_detailed as core_evaluate_batch_detailed,
    evaluate_batch_numeric as core_evaluate_batch_numeric,
    evaluate_batch_numeric_detailed as core_evaluate_batch_numeric_detailed,
    evaluate_numeric as core_evaluate_numeric, get_core_info as core_get_core_info,
    serialize as core_serialize, validate_rule as core_validate_rule,
};
use wasm_bindgen::prelude::*;

fn js_error(message: impl Into<String>) -> JsValue {
    JsValue::from_str(&message.into())
}

#[wasm_bindgen]
pub fn evaluate_wasm(rule_json: &str, context_json: &str) -> Result<String, JsValue> {
    let result = core_evaluate(rule_json, context_json).map_err(|e| js_error(e.to_string()))?;
    core_serialize(&result).map_err(|e| js_error(e.to_string()))
}

#[wasm_bindgen]
pub fn evaluate_numeric_wasm(rule_json: &str, context_json: &str) -> Result<f64, JsValue> {
    core_evaluate_numeric(rule_json, context_json).map_err(|e| js_error(e.to_string()))
}

#[wasm_bindgen]
pub fn evaluate_batch_wasm(rule_json: &str, contexts_json: &str) -> Result<String, JsValue> {
    let contexts: Vec<serde_json::Value> = serde_json::from_str(contexts_json)
        .map_err(|e| js_error(format!("Contexts parse error: {e}")))?;
    let serialized_contexts = contexts
        .iter()
        .map(core_serialize)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| js_error(e.to_string()))?;
    let result = core_evaluate_batch(rule_json, &serialized_contexts)
        .map_err(|e| js_error(e.to_string()))?;
    core_serialize(&result).map_err(|e| js_error(e.to_string()))
}

#[wasm_bindgen]
pub fn evaluate_batch_detailed_wasm(
    rule_json: &str,
    contexts_json: &str,
) -> Result<String, JsValue> {
    let contexts: Vec<serde_json::Value> = serde_json::from_str(contexts_json)
        .map_err(|e| js_error(format!("Contexts parse error: {e}")))?;
    let serialized_contexts = contexts
        .iter()
        .map(core_serialize)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| js_error(e.to_string()))?;
    let result = core_evaluate_batch_detailed(rule_json, &serialized_contexts)
        .map_err(|e| js_error(e.to_string()))?;
    core_serialize(&result).map_err(|e| js_error(e.to_string()))
}

#[wasm_bindgen]
pub fn evaluate_batch_numeric_wasm(
    rule_json: &str,
    contexts_json: &str,
) -> Result<String, JsValue> {
    let contexts: Vec<serde_json::Value> = serde_json::from_str(contexts_json)
        .map_err(|e| js_error(format!("Contexts parse error: {e}")))?;
    let serialized_contexts = contexts
        .iter()
        .map(core_serialize)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| js_error(e.to_string()))?;
    let result = core_evaluate_batch_numeric(rule_json, &serialized_contexts)
        .map_err(|e| js_error(e.to_string()))?;
    core_serialize(&result).map_err(|e| js_error(e.to_string()))
}

#[wasm_bindgen]
pub fn evaluate_batch_numeric_detailed_wasm(
    rule_json: &str,
    contexts_json: &str,
) -> Result<String, JsValue> {
    let contexts: Vec<serde_json::Value> = serde_json::from_str(contexts_json)
        .map_err(|e| js_error(format!("Contexts parse error: {e}")))?;
    let serialized_contexts = contexts
        .iter()
        .map(core_serialize)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| js_error(e.to_string()))?;
    let result = core_evaluate_batch_numeric_detailed(rule_json, &serialized_contexts)
        .map_err(|e| js_error(e.to_string()))?;
    core_serialize(&result).map_err(|e| js_error(e.to_string()))
}

#[wasm_bindgen]
pub fn validate_rule_wasm(rule_json: &str) -> Result<bool, JsValue> {
    core_validate_rule(rule_json).map_err(|e| js_error(e.to_string()))
}

#[wasm_bindgen]
pub fn get_core_info_wasm() -> String {
    core_serialize(&core_get_core_info()).unwrap_or_else(|_| "{}".to_string())
}
