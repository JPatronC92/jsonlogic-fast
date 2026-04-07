use serde_json::{Map, Value};

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
            value_type(&other)
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
            value_type(&other)
        ))),
    }
}

pub fn extract_array(result: Value) -> RuleEngineResult<Vec<Value>> {
    match result {
        Value::Array(values) => Ok(values),
        other => Err(RuleEngineError::NumericCoercion(format!(
            "Expected array result, got: {}",
            value_type(&other)
        ))),
    }
}

pub fn extract_object(result: Value) -> RuleEngineResult<Map<String, Value>> {
    match result {
        Value::Object(values) => Ok(values),
        other => Err(RuleEngineError::NumericCoercion(format!(
            "Expected object result, got: {}",
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
        if let RuleEngineError::NumericCoercion(msg) = error {
            assert!(
                !msg.contains("secret"),
                "Error message contains sensitive data: {}",
                msg
            );
        } else {
            panic!("Expected NumericCoercion error");
        }
    }
}
