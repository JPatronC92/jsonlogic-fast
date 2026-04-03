use thiserror::Error;

pub type RuleEngineResult<T> = Result<T, RuleEngineError>;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum RuleEngineError {
    #[error("Error parsing rule: {0}")]
    InvalidRule(String),

    #[error("Error parsing context: {0}")]
    InvalidContext(String),

    #[error("json-logic evaluation error: {0}")]
    Evaluation(String),

    #[error("Numeric coercion error: {0}")]
    NumericCoercion(String),

    #[error("Serialization error: {0}")]
    Serialization(String),
}
