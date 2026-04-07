use serde_json::Value;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_f64_accepts_bool_and_null() {
        assert_eq!(extract_f64(Value::Bool(true)).unwrap(), 1.0);
        assert_eq!(extract_f64(Value::Null).unwrap(), 0.0);
    }
}
