use serde_json::Value;

use crate::error::{RuleEngineError, RuleEngineResult};

fn value_type(v: &Value) -> &'static str {
    match v {
        Value::Null => "null",
        Value::Bool(_) => "boolean",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

pub fn extract_f64(result: Value) -> RuleEngineResult<f64> {
    match result {
        Value::Number(n) => n.as_f64().ok_or_else(|| {
            RuleEngineError::NumericCoercion(
                "Numeric result could not be converted to f64".to_string(),
            )
        }),
        Value::String(s) => s.parse::<f64>().map_err(|_| {
            RuleEngineError::NumericCoercion("String result is not a valid number".to_string())
        }),
        Value::Bool(b) => Ok(if b { 1.0 } else { 0.0 }),
        Value::Null => Ok(0.0),
        other => Err(RuleEngineError::NumericCoercion(format!(
            "Expected numeric result, got: {}",
            value_type(&other)
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn extract_f64_accepts_bool_and_null() {
        assert_eq!(extract_f64(Value::Bool(true)).unwrap(), 1.0);
        assert_eq!(extract_f64(Value::Null).unwrap(), 0.0);
    }

    #[test]
    fn test_extract_f64_exposure_protection() {
        let sensitive_data = json!({"secret": "sensitive"});
        let error = extract_f64(sensitive_data).unwrap_err();

        assert!(
            matches!(error, RuleEngineError::NumericCoercion(ref msg) if !msg.contains("secret")),
            "Expected NumericCoercion without sensitive data, but the error either had the wrong type or exposed the secret."
        );
    }

    #[test]
    fn test_extract_f64_errors() {
        let invalid_string = json!("abc");
        assert!(matches!(
            extract_f64(invalid_string),
            Err(RuleEngineError::NumericCoercion(_))
        ));

        let object = json!({"a": 1});
        assert!(matches!(
            extract_f64(object),
            Err(RuleEngineError::NumericCoercion(_))
        ));
    }
}
