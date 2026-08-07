use ::jsonlogic_fast::{
    evaluate as core_evaluate, evaluate_batch as core_evaluate_batch,
    evaluate_batch_detailed as core_evaluate_batch_detailed,
    evaluate_batch_numeric as core_evaluate_batch_numeric,
    evaluate_batch_numeric_detailed as core_evaluate_batch_numeric_detailed,
    evaluate_batch_numeric_strict as core_evaluate_batch_numeric_strict,
    evaluate_batch_strict as core_evaluate_batch_strict, evaluate_numeric as core_evaluate_numeric,
    get_core_info as core_get_core_info, validate_rule as core_validate_rule,
    CompiledRule as CoreCompiledRule,
};
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use serde_json::Value;

fn core_serialize(value: &impl serde::Serialize) -> PyResult<String> {
    serde_json::to_string(value)
        .map_err(|e| pyo3::exceptions::PyValueError::new_err(format!("Serialization error: {}", e)))
}

use pyo3::types::PyAny;
use pyo3::Py;
type PyObject = Py<PyAny>;

use pythonize::pythonize;

fn py_value_error(message: impl Into<String>) -> PyErr {
    PyValueError::new_err(message.into())
}

trait PyResultExt<T> {
    fn map_py_err(self) -> PyResult<T>;
}

impl<T, E: std::fmt::Display> PyResultExt<T> for Result<T, E> {
    fn map_py_err(self) -> PyResult<T> {
        self.map_err(|e| py_value_error(e.to_string()))
    }
}

#[pyfunction]
fn evaluate(py: Python<'_>, rule_json: &str, context_json: &str) -> PyResult<Py<PyAny>> {
    let result = core_evaluate(rule_json, context_json).map_py_err()?;
    pythonize(py, &result)
        .map(|value| value.unbind())
        .map_py_err()
}

#[pyfunction]
fn evaluate_rule(py: Python<'_>, rule_json: &str, context_json: &str) -> PyResult<Py<PyAny>> {
    evaluate(py, rule_json, context_json)
}

#[pyfunction]
fn evaluate_json(rule_json: &str, context_json: &str) -> PyResult<String> {
    let result = core_evaluate(rule_json, context_json).map_py_err()?;
    core_serialize(&result).map_py_err()
}

#[pyfunction]
fn evaluate_numeric(rule_json: &str, context_json: &str) -> PyResult<f64> {
    core_evaluate_numeric(rule_json, context_json).map_py_err()
}

#[pyfunction]
fn evaluate_batch(
    py: Python<'_>,
    rule_json: &str,
    contexts_json: Vec<String>,
) -> PyResult<Py<PyAny>> {
    let result = core_evaluate_batch(rule_json, &contexts_json).map_py_err()?;
    pythonize(py, &result)
        .map(|value| value.unbind())
        .map_py_err()
}

#[pyfunction]
fn evaluate_batch_json(rule_json: &str, contexts_json: Vec<String>) -> PyResult<String> {
    let result = core_evaluate_batch(rule_json, &contexts_json).map_py_err()?;
    core_serialize(&result).map_py_err()
}

#[pyfunction]
fn evaluate_batch_detailed(
    py: Python<'_>,
    rule_json: &str,
    contexts_json: Vec<String>,
) -> PyResult<Py<PyAny>> {
    let result = core_evaluate_batch_detailed(rule_json, &contexts_json).map_py_err()?;
    pythonize(py, &result)
        .map(|value| value.unbind())
        .map_py_err()
}

#[pyfunction]
fn evaluate_batch_numeric(rule_json: &str, contexts_json: Vec<String>) -> PyResult<Vec<f64>> {
    core_evaluate_batch_numeric(rule_json, &contexts_json).map_py_err()
}

#[pyfunction]
fn evaluate_batch_numeric_detailed(
    rule_json: &str,
    contexts_json: Vec<String>,
) -> PyResult<(Vec<f64>, Vec<String>)> {
    let result = core_evaluate_batch_numeric_detailed(rule_json, &contexts_json).map_py_err()?;

    let (values, errors) = result
        .into_iter()
        .map(|item| (item.result, item.error.unwrap_or_default()))
        .unzip();

    Ok((values, errors))
}

#[pyclass(module = "jsonlogic_fast", name = "CompiledRule")]
struct CompiledRule {
    core_rule: CoreCompiledRule,
}

#[pymethods]
impl CompiledRule {
    #[new]
    fn new(rule_json: &str) -> PyResult<Self> {
        let core_rule = CoreCompiledRule::new(rule_json).map_py_err()?;
        Ok(Self { core_rule })
    }

    fn evaluate(&self, py: Python<'_>, context_json: &str) -> PyResult<Py<PyAny>> {
        let result = self.core_rule.evaluate(context_json).map_py_err()?;
        pythonize(py, &result)
            .map(|value| value.unbind())
            .map_py_err()
    }

    fn evaluate_batch(&self, py: Python<'_>, contexts_json: Vec<String>) -> PyResult<Py<PyAny>> {
        let result = self.core_rule.evaluate_batch(&contexts_json).map_py_err()?;
        pythonize(py, &result)
            .map(|value| value.unbind())
            .map_py_err()
    }
}

#[pyfunction]
fn evaluate_batch_strict(
    py: Python<'_>,
    rule_json: &str,
    contexts_json: Vec<String>,
) -> PyResult<Py<PyAny>> {
    let result = core_evaluate_batch_strict(rule_json, &contexts_json).map_py_err()?;
    pythonize(py, &result)
        .map(|value| value.unbind())
        .map_py_err()
}

#[pyfunction]
fn evaluate_batch_numeric_strict(
    rule_json: &str,
    contexts_json: Vec<String>,
) -> PyResult<Vec<f64>> {
    core_evaluate_batch_numeric_strict(rule_json, &contexts_json).map_py_err()
}

#[pyfunction]
fn validate_rule(rule_json: &str) -> PyResult<bool> {
    core_validate_rule(rule_json).map_py_err()
}

#[pyfunction]
fn get_core_info() -> PyResult<String> {
    core_serialize(&core_get_core_info()).map_py_err()
}

// -----------------------------------------------------------------------------
// jsonlogic-ai: Policy, Guardrails, Router, Orchestrator
// -----------------------------------------------------------------------------

#[pyclass(name = "RuleEnvelope")]
#[derive(Clone)]
struct PyRuleEnvelope {
    inner: ::jsonlogic_fast::policy::RuleEnvelope,
}

#[pymethods]
impl PyRuleEnvelope {
    #[new]
    #[pyo3(signature = (id, assert_val, message, path=None, severity="error", retryable=true))]
    fn new(
        py: Python<'_>,
        id: String,
        assert_val: PyObject,
        message: String,
        path: Option<String>,
        severity: &str,
        retryable: bool,
    ) -> PyResult<Self> {
        let assert: Value = pythonize::depythonize(&assert_val.bind(py)).map_py_err()?;
        let severity_enum = match severity {
            "error" => ::jsonlogic_fast::policy::Severity::Error,
            "warning" => ::jsonlogic_fast::policy::Severity::Warning,
            _ => {
                return Err(py_value_error(
                    "Invalid severity value. Must be 'error' or 'warning'.",
                ))
            }
        };

        Ok(Self {
            inner: ::jsonlogic_fast::policy::RuleEnvelope {
                id,
                assert,
                message,
                path,
                severity: severity_enum,
                retryable,
            },
        })
    }

    #[getter]
    fn id(&self) -> String {
        self.inner.id.clone()
    }

    #[getter]
    fn assert_val(&self, py: Python<'_>) -> PyResult<PyObject> {
        pythonize(py, &self.inner.assert)
            .map(|v| v.unbind())
            .map_err(|e| py_value_error(e.to_string()))
    }

    #[getter]
    fn message(&self) -> String {
        self.inner.message.clone()
    }

    #[getter]
    fn path(&self) -> Option<String> {
        self.inner.path.clone()
    }

    #[getter]
    fn severity(&self) -> String {
        match self.inner.severity {
            ::jsonlogic_fast::policy::Severity::Error => "error".to_string(),
            ::jsonlogic_fast::policy::Severity::Warning => "warning".to_string(),
        }
    }

    #[getter]
    fn retryable(&self) -> bool {
        self.inner.retryable
    }
}

#[pyclass(name = "Violation")]
#[derive(Clone)]
struct PyViolation {
    inner: ::jsonlogic_fast::guardrails::Violation,
}

#[pymethods]
impl PyViolation {
    #[getter]
    fn rule_id(&self) -> String {
        self.inner.rule_id.clone()
    }

    #[getter]
    fn path(&self) -> Option<String> {
        self.inner.path.clone()
    }

    #[getter]
    fn message(&self) -> String {
        self.inner.message.clone()
    }

    #[getter]
    fn actual(&self, py: Python<'_>) -> PyResult<PyObject> {
        match &self.inner.actual {
            Some(v) => pythonize(py, v)
                .map(|val| val.unbind())
                .map_err(|e| py_value_error(e.to_string())),
            None => Ok(py.None()),
        }
    }

    #[getter]
    fn severity(&self) -> String {
        match self.inner.severity {
            ::jsonlogic_fast::policy::Severity::Error => "error".to_string(),
            ::jsonlogic_fast::policy::Severity::Warning => "warning".to_string(),
        }
    }

    #[getter]
    fn retryable(&self) -> bool {
        self.inner.retryable
    }
}

#[pyclass(name = "GuardrailReport")]
#[derive(Clone)]
struct PyGuardrailReport {
    inner: ::jsonlogic_fast::guardrails::GuardrailReport,
}

#[pymethods]
impl PyGuardrailReport {
    #[getter]
    fn valid(&self) -> bool {
        self.inner.valid
    }

    #[getter]
    fn violations(&self) -> Vec<PyViolation> {
        self.inner
            .violations
            .iter()
            .map(|v| PyViolation { inner: v.clone() })
            .collect()
    }
}

#[pyfunction]
fn evaluate_guardrails(
    py: Python<'_>,
    rules: Vec<PyRuleEnvelope>,
    output: PyObject,
) -> PyResult<PyGuardrailReport> {
    let core_rules: Vec<::jsonlogic_fast::policy::RuleEnvelope> =
        rules.into_iter().map(|r| r.inner).collect();

    let output_val: Value = pythonize::depythonize(&output.bind(py)).map_py_err()?;
    let report =
        ::jsonlogic_fast::guardrails::evaluate_guardrails(&core_rules, &output_val).map_py_err()?;

    Ok(PyGuardrailReport { inner: report })
}

#[pyclass(name = "Router")]
struct PyRouter {
    inner: ::jsonlogic_fast::router::Router,
}

#[pymethods]
impl PyRouter {
    #[new]
    fn new(py: Python<'_>, rule: PyObject) -> PyResult<Self> {
        let rule_val: Value = pythonize::depythonize(&rule.bind(py)).map_py_err()?;
        Ok(Self {
            inner: ::jsonlogic_fast::router::Router::new(rule_val),
        })
    }

    fn route(&self, py: Python<'_>, context: PyObject) -> PyResult<Py<PyAny>> {
        let context_val: Value = pythonize::depythonize(&context.bind(py)).map_py_err()?;
        let result = self.inner.route(&context_val).map_py_err()?;
        pythonize(py, &result)
            .map(|v| v.unbind())
            .map_err(|e| py_value_error(e.to_string()))
    }
}

#[pyclass(name = "AttemptLog")]
#[derive(Clone)]
struct PyAttemptLog {
    inner: ::jsonlogic_fast::orchestrator::AttemptLog,
}

#[pymethods]
impl PyAttemptLog {
    #[getter]
    fn attempt(&self) -> usize {
        self.inner.attempt
    }

    #[getter]
    fn raw_output(&self) -> String {
        self.inner.raw_output.clone()
    }

    #[getter]
    fn parsed_output(&self, py: Python<'_>) -> PyResult<PyObject> {
        match &self.inner.parsed_output {
            Some(v) => pythonize(py, v)
                .map(|val| val.unbind())
                .map_err(|e| py_value_error(e.to_string())),
            None => Ok(py.None()),
        }
    }

    #[getter]
    fn feedback(&self) -> Option<String> {
        self.inner.feedback.clone()
    }

    #[getter]
    fn valid(&self) -> bool {
        self.inner.valid
    }
}

#[pyclass(name = "OrchestrationResult")]
#[derive(Clone)]
struct PyOrchestrationResult {
    inner: ::jsonlogic_fast::orchestrator::OrchestrationResult,
}

#[pymethods]
impl PyOrchestrationResult {
    #[getter]
    fn success(&self) -> bool {
        self.inner.success
    }

    #[getter]
    fn attempts(&self) -> Vec<PyAttemptLog> {
        self.inner
            .attempts
            .iter()
            .map(|a| PyAttemptLog { inner: a.clone() })
            .collect()
    }

    #[getter]
    fn output(&self, py: Python<'_>) -> PyResult<PyObject> {
        match &self.inner.output {
            Some(v) => pythonize(py, v)
                .map(|val| val.unbind())
                .map_err(|e| py_value_error(e.to_string())),
            None => Ok(py.None()),
        }
    }

    #[getter]
    fn error_message(&self) -> Option<String> {
        self.inner.error_message.clone()
    }
}

#[pyfunction]
fn orchestrate_reflection(
    py: Python<'_>,
    rules: Vec<PyRuleEnvelope>,
    generate_fn: PyObject,
    initial_messages: Vec<PyObject>,
    max_retries: usize,
) -> PyResult<PyOrchestrationResult> {
    let core_rules: Vec<::jsonlogic_fast::policy::RuleEnvelope> =
        rules.into_iter().map(|r| r.inner).collect();

    let mut core_messages = Vec::new();
    for msg in initial_messages {
        let val: Value = pythonize::depythonize(&msg.bind(py)).map_py_err()?;
        core_messages.push(val);
    }

    let generate_wrapper =
        |messages: &[Value]| -> Result<String, ::jsonlogic_fast::error::RuleEngineError> {
            let mut py_messages = Vec::with_capacity(messages.len());
            for m in messages {
                let py_val = pythonize(py, m).map_err(|e| {
                    ::jsonlogic_fast::error::RuleEngineError::Evaluation(format!(
                        "Pythonize error: {}",
                        e
                    ))
                })?;
                py_messages.push(py_val.unbind());
            }

            let res = generate_fn.call1(py, (py_messages,)).map_err(|e| {
                ::jsonlogic_fast::error::RuleEngineError::Evaluation(format!(
                    "Python generate_fn error: {:?}",
                    e
                ))
            })?;

            let raw_output: String = res.extract(py).map_err(|e| {
                ::jsonlogic_fast::error::RuleEngineError::Evaluation(format!(
                    "Python generate_fn must return a string: {:?}",
                    e
                ))
            })?;

            Ok(raw_output)
        };

    let result = ::jsonlogic_fast::orchestrator::orchestrate_reflection(
        &core_rules,
        generate_wrapper,
        &core_messages,
        max_retries,
    )
    .map_py_err()?;

    Ok(PyOrchestrationResult { inner: result })
}

#[pymodule]
fn jsonlogic_fast(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(evaluate, m)?)?;
    m.add_function(wrap_pyfunction!(evaluate_rule, m)?)?;
    m.add_function(wrap_pyfunction!(evaluate_json, m)?)?;
    m.add_function(wrap_pyfunction!(evaluate_numeric, m)?)?;
    m.add_function(wrap_pyfunction!(evaluate_batch, m)?)?;
    m.add_function(wrap_pyfunction!(evaluate_batch_json, m)?)?;
    m.add_function(wrap_pyfunction!(evaluate_batch_detailed, m)?)?;
    m.add_function(wrap_pyfunction!(evaluate_batch_numeric, m)?)?;
    m.add_function(wrap_pyfunction!(evaluate_batch_numeric_detailed, m)?)?;
    m.add_class::<CompiledRule>()?;
    m.add_function(wrap_pyfunction!(evaluate_batch_strict, m)?)?;
    m.add_function(wrap_pyfunction!(evaluate_batch_numeric_strict, m)?)?;
    m.add_function(wrap_pyfunction!(validate_rule, m)?)?;
    m.add_function(wrap_pyfunction!(get_core_info, m)?)?;

    // jsonlogic-ai classes
    m.add_class::<PyRuleEnvelope>()?;
    m.add_class::<PyViolation>()?;
    m.add_class::<PyGuardrailReport>()?;
    m.add_class::<PyRouter>()?;
    m.add_class::<PyAttemptLog>()?;
    m.add_class::<PyOrchestrationResult>()?;

    // jsonlogic-ai functions
    m.add_function(wrap_pyfunction!(evaluate_guardrails, m)?)?;
    m.add_function(wrap_pyfunction!(orchestrate_reflection, m)?)?;
    m.add_function(wrap_pyfunction!(compile_dsl, m)?)?;

    Ok(())
}

#[pyfunction]
fn compile_dsl(py: Python<'_>, dsl_str: &str) -> PyResult<PyObject> {
    let result = ::jsonlogic_fast::compile_dsl(dsl_str).map_py_err()?;
    pythonize(py, &result)
        .map(|v| v.unbind())
        .map_err(|e| py_value_error(e.to_string()))
}
