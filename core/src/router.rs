use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::{RuleEngineError, RuleEngineResult};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Router {
    pub rule: Value,
}

impl Router {
    pub fn new(rule: Value) -> Self {
        Self { rule }
    }

    pub fn route(&self, context: &Value) -> RuleEngineResult<Value> {
        jsonlogic::apply(&self.rule, context)
            .map_err(|e| RuleEngineError::Evaluation(format!("Router evaluation error: {}", e)))
    }
}
