use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::{RuleEngineError, RuleEngineResult};
use crate::policy::{RuleEnvelope, Severity};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Violation {
    pub rule_id: String,
    pub path: Option<String>,
    pub message: String,
    pub actual: Option<Value>,
    pub severity: Severity,
    pub retryable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GuardrailReport {
    pub valid: bool,
    pub violations: Vec<Violation>,
}

/// Helper function to extract a value from context based on a simple path/pointer.
pub fn extract_actual(value: &Value, path: &str) -> Option<Value> {
    if path.starts_with('/') {
        value.pointer(path).cloned()
    } else {
        let clean_path = if let Some(stripped) = path.strip_prefix("$.") {
            stripped
        } else if let Some(stripped) = path.strip_prefix('$') {
            stripped
        } else {
            path
        };
        if clean_path.is_empty() {
            return Some(value.clone());
        }
        let parts: Vec<&str> = clean_path.split('.').collect();
        let mut current = value;
        for part in parts {
            if let Some(next) = current.get(part) {
                current = next;
            } else {
                return None;
            }
        }
        Some(current.clone())
    }
}

/// Check if a serde_json::Value is truthy in JSON-Logic terms.
pub fn is_truthy(v: &Value) -> bool {
    match v {
        Value::Null => false,
        Value::Bool(b) => *b,
        Value::Number(n) => {
            if let Some(f) = n.as_f64() {
                f != 0.0
            } else if let Some(i) = n.as_i64() {
                i != 0
            } else if let Some(u) = n.as_u64() {
                u != 0
            } else {
                false
            }
        }
        Value::String(s) => !s.is_empty(),
        Value::Array(arr) => !arr.is_empty(),
        Value::Object(_) => true,
    }
}

/// Evaluate structured data against a set of guardrail policies.
/// Returns a top-level RuleEngineError::Evaluation error if any rule fails to compile or evaluate (EvaluationError).
/// Returns Ok(GuardrailReport) with violations for any rule that evaluated to falsy (PolicyViolation).
pub fn evaluate_guardrails(
    rules: &[RuleEnvelope],
    output: &Value,
) -> RuleEngineResult<GuardrailReport> {
    let mut violations = Vec::new();

    for rule in rules {
        // Evaluate the rule.assert using jsonlogic::apply with the output as the context
        let result = jsonlogic::apply(&rule.assert, output).map_err(|e| {
            RuleEngineError::Evaluation(format!("Evaluation error in rule '{}': {}", rule.id, e))
        })?;

        if !is_truthy(&result) {
            let actual = rule.path.as_ref().and_then(|p| extract_actual(output, p));

            violations.push(Violation {
                rule_id: rule.id.clone(),
                path: rule.path.clone(),
                message: rule.message.clone(),
                actual,
                severity: rule.severity,
                retryable: rule.retryable,
            });
        }
    }

    let valid = !violations.iter().any(|v| v.severity == Severity::Error);
    Ok(GuardrailReport { valid, violations })
}
