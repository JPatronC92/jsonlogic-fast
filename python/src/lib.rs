use ::jsonlogic_fast::{
    evaluate as core_evaluate, evaluate_batch as core_evaluate_batch,
    evaluate_batch_detailed as core_evaluate_batch_detailed,
    evaluate_batch_numeric as core_evaluate_batch_numeric,
    evaluate_batch_numeric_detailed as core_evaluate_batch_numeric_detailed,
    evaluate_numeric as core_evaluate_numeric, get_core_info as core_get_core_info,
    serialize as core_serialize, validate_rule as core_validate_rule,
};
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::PyAny;
use pythonize::pythonize;

fn py_value_error(message: impl Into<String>) -> PyErr {
    PyValueError::new_err(message.into())
}

#[pyfunction]
fn evaluate(py: Python<'_>, rule_json: &str, context_json: &str) -> PyResult<Py<PyAny>> {
    let result =
        core_evaluate(rule_json, context_json).map_err(|e| py_value_error(e.to_string()))?;
    pythonize(py, &result)
        .map(|value| value.unbind())
        .map_err(|e| py_value_error(e.to_string()))
}

#[pyfunction]
fn evaluate_rule(py: Python<'_>, rule_json: &str, context_json: &str) -> PyResult<Py<PyAny>> {
    evaluate(py, rule_json, context_json)
}

#[pyfunction]
fn evaluate_json(rule_json: &str, context_json: &str) -> PyResult<String> {
    let result =
        core_evaluate(rule_json, context_json).map_err(|e| py_value_error(e.to_string()))?;
    core_serialize(&result).map_err(|e| py_value_error(e.to_string()))
}

#[pyfunction]
fn evaluate_numeric(rule_json: &str, context_json: &str) -> PyResult<f64> {
    core_evaluate_numeric(rule_json, context_json).map_err(|e| py_value_error(e.to_string()))
}

#[pyfunction]
fn evaluate_batch(
    py: Python<'_>,
    rule_json: &str,
    contexts_json: Vec<String>,
) -> PyResult<Py<PyAny>> {
    let result = core_evaluate_batch(rule_json, &contexts_json)
        .map_err(|e| py_value_error(e.to_string()))?;
    pythonize(py, &result)
        .map(|value| value.unbind())
        .map_err(|e| py_value_error(e.to_string()))
}

#[pyfunction]
fn evaluate_batch_json(rule_json: &str, contexts_json: Vec<String>) -> PyResult<String> {
    let result = core_evaluate_batch(rule_json, &contexts_json)
        .map_err(|e| py_value_error(e.to_string()))?;
    core_serialize(&result).map_err(|e| py_value_error(e.to_string()))
}

#[pyfunction]
fn evaluate_batch_detailed(
    py: Python<'_>,
    rule_json: &str,
    contexts_json: Vec<String>,
) -> PyResult<Py<PyAny>> {
    let result = core_evaluate_batch_detailed(rule_json, &contexts_json)
        .map_err(|e| py_value_error(e.to_string()))?;
    pythonize(py, &result)
        .map(|value| value.unbind())
        .map_err(|e| py_value_error(e.to_string()))
}

#[pyfunction]
fn evaluate_batch_numeric(rule_json: &str, contexts_json: Vec<String>) -> PyResult<Vec<f64>> {
    core_evaluate_batch_numeric(rule_json, &contexts_json)
        .map_err(|e| py_value_error(e.to_string()))
}

#[pyfunction]
fn evaluate_batch_numeric_detailed(
    rule_json: &str,
    contexts_json: Vec<String>,
) -> PyResult<(Vec<f64>, Vec<String>)> {
    let result = core_evaluate_batch_numeric_detailed(rule_json, &contexts_json)
        .map_err(|e| py_value_error(e.to_string()))?;

    let values = result.iter().map(|item| item.result).collect();
    let errors = result
        .into_iter()
        .map(|item| item.error.unwrap_or_default())
        .collect();

    Ok((values, errors))
}

#[pyfunction]
fn validate_rule(rule_json: &str) -> PyResult<bool> {
    core_validate_rule(rule_json).map_err(|e| py_value_error(e.to_string()))
}

#[pyfunction]
fn get_core_info() -> PyResult<String> {
    core_serialize(&core_get_core_info()).map_err(|e| py_value_error(e.to_string()))
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
