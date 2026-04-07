use serde_json::{Map, Value};

use crate::error::{RuleEngineError, RuleEngineResult};

pub fn extract_f64(result: Value) -> RuleEngineResult<f64> {
    match result {
        Value::Number(n) => n.as_f64().ok_or_else(|| {
            RuleEngineError::NumericCoercion(
                "Numeric result could not be converted to f64".to_string(),
            )
        }),
        Value::String(s) => s.parse::<f64>().map_err(|_| {
            RuleEngineError::NumericCoercion(format!("String result '{}' is not a valid number", s))
        }),
        Value::Bool(b) => Ok(if b { 1.0 } else { 0.0 }),
        Value::Null => Ok(0.0),
        other => Err(RuleEngineError::NumericCoercion(format!(
            "Expected numeric result, got: {}",
            serde_json::to_string(&other).unwrap_or_default()
        ))),
    }
}

pub fn extract_bool(result: Value) -> RuleEngineResult<bool> {
    match result {
        Value::Bool(value) => Ok(value),
        Value::Number(n) => Ok(n.as_f64().unwrap_or_default() != 0.0),
        Value::String(s) => match s.to_ascii_lowercase().as_str() {
            "true" | "1" => Ok(true),
            "false" | "0" | "" => Ok(false),
            _ => Err(RuleEngineError::NumericCoercion(format!(
                "String result '{}' is not a valid boolean",
                s
            ))),
        },
        Value::Null => Ok(false),
        other => Err(RuleEngineError::NumericCoercion(format!(
            "Expected boolean result, got: {}",
            serde_json::to_string(&other).unwrap_or_default()
        ))),
    }
}

pub fn extract_string(result: Value) -> RuleEngineResult<String> {
    match result {
        Value::String(value) => Ok(value),
        Value::Number(n) => Ok(n.to_string()),
        Value::Bool(b) => Ok(b.to_string()),
        Value::Null => Ok(String::new()),
        other => Err(RuleEngineError::NumericCoercion(format!(
            "Expected string result, got: {}",
            serde_json::to_string(&other).unwrap_or_default()
        ))),
    }
}

pub fn extract_array(result: Value) -> RuleEngineResult<Vec<Value>> {
    match result {
        Value::Array(values) => Ok(values),
        other => Err(RuleEngineError::NumericCoercion(format!(
            "Expected array result, got: {}",
            serde_json::to_string(&other).unwrap_or_default()
        ))),
    }
}

pub fn extract_object(result: Value) -> RuleEngineResult<Map<String, Value>> {
    match result {
        Value::Object(values) => Ok(values),
        other => Err(RuleEngineError::NumericCoercion(format!(
            "Expected object result, got: {}",
            serde_json::to_string(&other).unwrap_or_default()
        ))),
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn extract_f64_accepts_bool_and_null() {
        assert_eq!(extract_f64(Value::Bool(true)).unwrap(), 1.0);
        assert_eq!(extract_f64(Value::Null).unwrap(), 0.0);
    }

    #[test]
    fn extract_bool_accepts_common_scalar_values() {
        assert!(extract_bool(Value::String("true".to_string())).unwrap());
        assert!(!extract_bool(json!(0)).unwrap());
    }

    #[test]
    fn extract_string_serializes_scalars() {
        assert_eq!(extract_string(json!(7)).unwrap(), "7");
        assert_eq!(extract_string(Value::Bool(false)).unwrap(), "false");
    }

    #[test]
    fn extract_array_rejects_non_arrays() {
        let error = extract_array(json!({"a": 1})).unwrap_err();
        assert!(matches!(error, RuleEngineError::NumericCoercion(_)));
    }

    #[test]
    fn extract_object_accepts_json_objects() {
        let object = extract_object(json!({"country": "MX"})).unwrap();
        assert_eq!(object.get("country"), Some(&json!("MX")));
    }

    #[test]
    fn extract_bool_rejects_invalid_string() {
        let error = extract_bool(Value::String("yes".to_string())).unwrap_err();
        assert!(matches!(error, RuleEngineError::NumericCoercion(_)));
    }
}
