use jsonlogic_fast::{
    evaluate as core_evaluate, evaluate_batch as core_evaluate_batch,
    evaluate_batch_detailed as core_evaluate_batch_detailed,
    evaluate_batch_numeric as core_evaluate_batch_numeric,
    evaluate_batch_numeric_detailed as core_evaluate_batch_numeric_detailed,
    evaluate_numeric as core_evaluate_numeric, get_core_info as core_get_core_info,
    validate_rule as core_validate_rule,
};
use wasm_bindgen::prelude::*;

fn core_serialize(
    value: &impl serde::Serialize,
) -> Result<String, jsonlogic_fast::error::RuleEngineError> {
    serde_json::to_string(value)
        .map_err(|e| jsonlogic_fast::error::RuleEngineError::Serialization(e.to_string()))
}

fn js_error(message: impl Into<String>) -> JsValue {
    JsValue::from_str(&message.into())
}

trait JsResultExt<T> {
    fn map_js_err(self) -> Result<T, JsValue>;
    fn map_js_err_with_prefix(self, prefix: &str) -> Result<T, JsValue>;
}

impl<T, E: std::fmt::Display> JsResultExt<T> for Result<T, E> {
    fn map_js_err(self) -> Result<T, JsValue> {
        self.map_err(|e| js_error(e.to_string()))
    }

    fn map_js_err_with_prefix(self, prefix: &str) -> Result<T, JsValue> {
        self.map_err(|e| js_error(format!("{prefix}: {e}")))
    }
}

#[wasm_bindgen]
pub fn evaluate_wasm(rule_json: &str, context_json: &str) -> Result<String, JsValue> {
    let result = core_evaluate(rule_json, context_json).map_js_err()?;
    core_serialize(&result).map_js_err()
}

#[wasm_bindgen]
pub fn evaluate_numeric_wasm(rule_json: &str, context_json: &str) -> Result<f64, JsValue> {
    core_evaluate_numeric(rule_json, context_json).map_js_err()
}

#[wasm_bindgen]
pub fn evaluate_batch_wasm(rule_json: &str, contexts_json: &str) -> Result<String, JsValue> {
    let contexts: Vec<serde_json::Value> =
        serde_json::from_str(contexts_json).map_js_err_with_prefix("Contexts parse error")?;
    let serialized_contexts = contexts
        .iter()
        .map(core_serialize)
        .collect::<Result<Vec<_>, _>>()
        .map_js_err()?;
    let result = core_evaluate_batch(rule_json, &serialized_contexts).map_js_err()?;
    core_serialize(&result).map_js_err()
}

#[wasm_bindgen]
pub fn evaluate_batch_detailed_wasm(
    rule_json: &str,
    contexts_json: &str,
) -> Result<String, JsValue> {
    let contexts: Vec<serde_json::Value> =
        serde_json::from_str(contexts_json).map_js_err_with_prefix("Contexts parse error")?;
    let serialized_contexts = contexts
        .iter()
        .map(core_serialize)
        .collect::<Result<Vec<_>, _>>()
        .map_js_err()?;
    let result = core_evaluate_batch_detailed(rule_json, &serialized_contexts).map_js_err()?;
    core_serialize(&result).map_js_err()
}

#[wasm_bindgen]
pub fn evaluate_batch_numeric_wasm(
    rule_json: &str,
    contexts_json: &str,
) -> Result<String, JsValue> {
    let contexts: Vec<serde_json::Value> =
        serde_json::from_str(contexts_json).map_js_err_with_prefix("Contexts parse error")?;
    let serialized_contexts = contexts
        .iter()
        .map(core_serialize)
        .collect::<Result<Vec<_>, _>>()
        .map_js_err()?;
    let result = core_evaluate_batch_numeric(rule_json, &serialized_contexts).map_js_err()?;
    core_serialize(&result).map_js_err()
}

#[wasm_bindgen]
pub fn evaluate_batch_numeric_detailed_wasm(
    rule_json: &str,
    contexts_json: &str,
) -> Result<String, JsValue> {
    let contexts: Vec<serde_json::Value> =
        serde_json::from_str(contexts_json).map_js_err_with_prefix("Contexts parse error")?;
    let serialized_contexts = contexts
        .iter()
        .map(core_serialize)
        .collect::<Result<Vec<_>, _>>()
        .map_js_err()?;
    let result =
        core_evaluate_batch_numeric_detailed(rule_json, &serialized_contexts).map_js_err()?;
    core_serialize(&result).map_js_err()
}

#[wasm_bindgen]
pub fn validate_rule_wasm(rule_json: &str) -> Result<bool, JsValue> {
    core_validate_rule(rule_json).map_js_err()
}

#[wasm_bindgen]
pub fn get_core_info_wasm() -> String {
    core_serialize(&core_get_core_info()).unwrap_or_else(|_| "{}".to_string())
}
