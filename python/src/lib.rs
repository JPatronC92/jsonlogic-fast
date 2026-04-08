use ::jsonlogic_fast::{
    evaluate as core_evaluate, evaluate_batch as core_evaluate_batch,
    evaluate_batch_detailed as core_evaluate_batch_detailed,
    evaluate_batch_numeric as core_evaluate_batch_numeric,
    evaluate_batch_numeric_detailed as core_evaluate_batch_numeric_detailed,
    evaluate_numeric as core_evaluate_numeric, get_core_info as core_get_core_info,
    validate_rule as core_validate_rule,
};
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

fn core_serialize(value: &impl serde::Serialize) -> PyResult<String> {
    serde_json::to_string(value)
        .map_err(|e| pyo3::exceptions::PyValueError::new_err(format!("Serialization error: {}", e)))
}

use pyo3::types::PyAny;
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

#[pyfunction]
fn validate_rule(rule_json: &str) -> PyResult<bool> {
    core_validate_rule(rule_json).map_py_err()
}

#[pyfunction]
fn get_core_info() -> PyResult<String> {
    core_serialize(&core_get_core_info()).map_py_err()
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
    m.add_function(wrap_pyfunction!(validate_rule, m)?)?;
    m.add_function(wrap_pyfunction!(get_core_info, m)?)?;
    Ok(())
}
