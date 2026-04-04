//! # jsonlogic-fast
//!
//! **Fast, embeddable, cross-runtime JSON-Logic evaluation.**
//!
//! Implements the full [JsonLogic](https://jsonlogic.com/) specification with
//! batch evaluation, numeric coercion, and parallel execution via Rayon
//! (sequential on WASM).
//!
//! ## Quick start
//!
//! ```rust
//! use jsonlogic_fast::evaluate;
//!
//! let rule = r#"{">":[{"var":"age"},18]}"#;
//! let context = r#"{"age":25}"#;
//! let result = evaluate(rule, context).unwrap();
//! assert_eq!(result, serde_json::json!(true));
//! ```
//!
//! ## Batch evaluation
//!
//! ```rust
//! use jsonlogic_fast::evaluate_batch;
//!
//! let rule = r#"{"var":"score"}"#;
//! let contexts = vec![
//!     r#"{"score":90}"#.to_string(),
//!     r#"{"score":45}"#.to_string(),
//! ];
//! let results = evaluate_batch(rule, &contexts).unwrap();
//! assert_eq!(results, vec![serde_json::json!(90), serde_json::json!(45)]);
//! ```

//! # jsonlogic-fast
//!
//! **Fast, embeddable, cross-runtime JSON-Logic evaluation.**
//!
//! Implements the full [JsonLogic](https://jsonlogic.com/) specification with
//! batch evaluation, numeric coercion, and parallel execution via Rayon
//! (sequential on WASM).
//!
//! ## Quick start
//!
//! ```rust
//! use jsonlogic_fast::evaluate;
//!
//! let rule = r#"{">":[{"var":"age"},18]}"#;
//! let context = r#"{"age":25}"#;
//! let result = evaluate(rule, context).unwrap();
//! assert_eq!(result, serde_json::json!(true));
//! ```
//!
//! ## Batch evaluation
//!
//! ```rust
//! use jsonlogic_fast::evaluate_batch;
//!
//! let rule = r#"{"var":"score"}"#;
//! let contexts = vec![
//!     r#"{"score":90}"#.to_string(),
//!     r#"{"score":45}"#.to_string(),
//! ];
//! let results = evaluate_batch(rule, &contexts).unwrap();
//! assert_eq!(results, vec![serde_json::json!(90), serde_json::json!(45)]);
//! ```

pub mod error;
pub mod extract;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use error::{RuleEngineError, RuleEngineResult};
use extract::extract_f64;

#[cfg(not(target_arch = "wasm32"))]
use rayon::prelude::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
/// Result of a single rule evaluation with optional error.
/// Result of a single rule evaluation with optional error.
pub struct EvaluationResult {
    pub result: Option<Value>,
    pub error: Option<String>,
}

impl EvaluationResult {
    fn ok(result: Value) -> Self {
        Self {
            result: Some(result),
            error: None,
        }
    }

    fn err(error: impl Into<String>) -> Self {
        Self {
            result: None,
            error: Some(error.into()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
/// Numeric evaluation result with optional error.
/// Numeric evaluation result with optional error.
pub struct NumericEvaluationResult {
    pub result: f64,
    pub error: Option<String>,
}

fn parse_rule(rule_json: &str) -> RuleEngineResult<Value> {
    serde_json::from_str(rule_json).map_err(|e| RuleEngineError::InvalidRule(e.to_string()))
}

fn parse_context(context_json: &str) -> RuleEngineResult<Value> {
    serde_json::from_str(context_json).map_err(|e| RuleEngineError::InvalidContext(e.to_string()))
}

fn apply_rule(rule: &Value, context: &Value) -> RuleEngineResult<Value> {
    jsonlogic::apply(rule, context).map_err(|e| RuleEngineError::Evaluation(e.to_string()))
}

fn evaluate_context(rule: &Value, context_json: &str) -> EvaluationResult {
    let context = match parse_context(context_json) {
        Ok(context) => context,
        Err(error) => return EvaluationResult::err(error.to_string()),
    };

    match apply_rule(rule, &context) {
        Ok(result) => EvaluationResult::ok(result),
        Err(error) => EvaluationResult::err(error.to_string()),
    }
}

fn default_validation_context() -> Value {
    json!({
        "amount": 100.0,
        "country": "MX",
        "method": "CREDIT_CARD",
        "active": true,
        "count": 3,
        "score": 720,
        "tags": ["vip", "beta"],
        "user": {
            "tier": "gold",
            "region": "north"
        },
        "metrics": {
            "total_volume": 1000.0,
            "chargebacks": 1
        },
        "null_value": null
    })
}

#[cfg(not(target_arch = "wasm32"))]
fn map_contexts<T, F>(contexts_json: &[String], evaluator: F) -> Vec<T>
where
    T: Send,
    F: Fn(&str) -> T + Sync + Send,
{
    contexts_json
        .par_iter()
        .map(|context_json| evaluator(context_json))
        .collect()
}

#[cfg(target_arch = "wasm32")]
fn map_contexts<T, F>(contexts_json: &[String], evaluator: F) -> Vec<T>
where
    F: Fn(&str) -> T,
{
    contexts_json
        .iter()
        .map(|context_json| evaluator(context_json))
        .collect()
}

#[cfg(not(target_arch = "wasm32"))]
fn available_threads() -> usize {
    rayon::current_num_threads()
}

#[cfg(target_arch = "wasm32")]
fn available_threads() -> usize {
    1
}

/// Evaluate a JSON-Logic rule against a single JSON context.
/// Evaluate a JSON-Logic rule against a single JSON context.
pub fn evaluate(rule_json: &str, context_json: &str) -> RuleEngineResult<Value> {
    let rule = parse_rule(rule_json)?;
    let context = parse_context(context_json)?;
    apply_rule(&rule, &context)
}

/// Alias for [`evaluate`].
/// Alias for [`evaluate`].
pub fn evaluate_rule(rule_json: &str, context_json: &str) -> RuleEngineResult<Value> {
    evaluate(rule_json, context_json)
}

/// Evaluate a rule and coerce the result to `f64`.
/// Evaluate a rule and coerce the result to `f64`.
pub fn evaluate_numeric(rule_json: &str, context_json: &str) -> RuleEngineResult<f64> {
    extract_f64(evaluate(rule_json, context_json)?)
}

/// Evaluate a rule against many contexts (parallel on native, sequential on WASM).
/// Evaluate a rule against many contexts (parallel on native, sequential on WASM).
pub fn evaluate_batch(rule_json: &str, contexts_json: &[String]) -> RuleEngineResult<Vec<Value>> {
    let rule = parse_rule(rule_json)?;
    Ok(map_contexts(contexts_json, |context_json| {
        evaluate_context(&rule, context_json)
            .result
            .unwrap_or(Value::Null)
    }))
}

/// Like [`evaluate_batch`] but returns [`EvaluationResult`] with error details.
/// Like [`evaluate_batch`] but returns [`EvaluationResult`] with error details.
pub fn evaluate_batch_detailed(
    rule_json: &str,
    contexts_json: &[String],
) -> RuleEngineResult<Vec<EvaluationResult>> {
    let rule = parse_rule(rule_json)?;
    Ok(map_contexts(contexts_json, |context_json| {
        evaluate_context(&rule, context_json)
    }))
}

/// Batch-evaluate and coerce every result to `f64`.
/// Batch-evaluate and coerce every result to `f64`.
pub fn evaluate_batch_numeric(
    rule_json: &str,
    contexts_json: &[String],
) -> RuleEngineResult<Vec<f64>> {
    let results = evaluate_batch_detailed(rule_json, contexts_json)?;
    Ok(results
        .into_iter()
        .map(|item| match item.result {
            Some(result) => extract_f64(result).unwrap_or(0.0),
            None => 0.0,
        })
        .collect())
}

/// Like [`evaluate_batch_numeric`] but returns [`NumericEvaluationResult`] with errors.
/// Like [`evaluate_batch_numeric`] but returns [`NumericEvaluationResult`] with errors.
pub fn evaluate_batch_numeric_detailed(
    rule_json: &str,
    contexts_json: &[String],
) -> RuleEngineResult<Vec<NumericEvaluationResult>> {
    let results = evaluate_batch_detailed(rule_json, contexts_json)?;
    Ok(results
        .into_iter()
        .map(|item| match (item.result, item.error) {
            (Some(result), None) => match extract_f64(result) {
                Ok(value) => NumericEvaluationResult {
                    result: value,
                    error: None,
                },
                Err(error) => NumericEvaluationResult {
                    result: 0.0,
                    error: Some(error.to_string()),
                },
            },
            (_, Some(error)) => NumericEvaluationResult {
                result: 0.0,
                error: Some(error),
            },
            (None, None) => NumericEvaluationResult {
                result: 0.0,
                error: Some("Unknown evaluation failure".to_string()),
            },
        })
        .collect())
}

/// Validate that a rule can be evaluated without errors.
/// Validate that a rule can be evaluated without errors.
pub fn validate_rule(rule_json: &str) -> RuleEngineResult<bool> {
    let rule = parse_rule(rule_json)?;
    let context = default_validation_context();

    apply_rule(&rule, &context)
        .map(|_| true)
        .map_err(|error| RuleEngineError::Evaluation(format!("Rule validation failed: {error}")))
}

/// Serialize a [`Value`] to a JSON string.
/// Serialize a [`Value`] to a JSON string.
pub fn serialize_value(value: &Value) -> RuleEngineResult<String> {
    serde_json::to_string(value).map_err(|e| RuleEngineError::Serialization(e.to_string()))
}

/// Serialize any `Serialize` implementor to a JSON string.
/// Serialize any `Serialize` implementor to a JSON string.
pub fn serialize<T: Serialize>(value: &T) -> RuleEngineResult<String> {
    serde_json::to_string(value).map_err(|e| RuleEngineError::Serialization(e.to_string()))
}

/// Return engine metadata (version, parallelism mode, thread count).
/// Return engine metadata (version, parallelism mode, thread count).
pub fn get_core_info() -> Value {
    json!({
        "engine": "jsonlogic-fast",
        "version": env!("CARGO_PKG_VERSION"),
        "parallelism": if cfg!(target_arch = "wasm32") { "sequential" } else { "rayon" },
        "available_threads": available_threads(),
        "evaluator": "jsonlogic-rs",
        "result_model": "serde_json::Value"
    })
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn evaluate_preserves_string_results() {
        let rule = r#"{"if":[{"==":[{"var":"country"},"MX"]},"domestic","intl"]}"#;
        let context = r#"{"country":"MX"}"#;

        assert_eq!(evaluate(rule, context).unwrap(), json!("domestic"));
    }

    #[test]
    fn evaluate_numeric_coerces_boolean_results() {
        let rule = r#"{"==":[{"var":"country"},"MX"]}"#;
        let context = r#"{"country":"MX"}"#;

        assert_eq!(evaluate_numeric(rule, context).unwrap(), 1.0);
    }

    #[test]
    fn evaluate_batch_returns_null_for_invalid_contexts() {
        let rule = r#"{"var":"amount"}"#;
        let contexts = vec![r#"{"amount":10}"#.to_string(), "{bad json}".to_string()];

        assert_eq!(
            evaluate_batch(rule, &contexts).unwrap(),
            vec![json!(10), Value::Null]
        );
    }

    #[test]
    fn evaluate_batch_detailed_reports_errors() {
        let rule = r#"{"var":"amount"}"#;
        let contexts = vec!["{}".to_string(), "{bad json}".to_string()];
        let results = evaluate_batch_detailed(rule, &contexts).unwrap();

        assert_eq!(results[0].result, Some(Value::Null));
        assert!(results[1]
            .error
            .as_ref()
            .is_some_and(|message| message.contains("Error parsing context")));
    }

    #[test]
    fn evaluate_batch_numeric_keeps_fail_safe_zeroes() {
        let rule = r#"{"var":"amount"}"#;
        let contexts = vec!["{}".to_string(), "{bad json}".to_string()];

        assert_eq!(
            evaluate_batch_numeric(rule, &contexts).unwrap(),
            vec![0.0, 0.0]
        );
    }

    #[test]
    fn validate_rule_accepts_generic_contexts() {
        let rule = r#"{"cat":[{"var":"user.tier"},"-",{"var":"country"}]}"#;
        assert!(validate_rule(rule).unwrap());
    }
}
