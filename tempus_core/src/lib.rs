use pyo3::prelude::*;
use pyo3::exceptions::PyValueError;
use serde_json::Value;
use jsonlogic;

/// Evalúa una regla matemática usando json-logic en Rust.
/// Recibe la regla y el contexto como cadenas JSON y devuelve el monto (f64).
#[pyfunction]
fn evaluate_fee(rule_json: &str, context_json: &str) -> PyResult<f64> {
    // 1. Parsear los JSONs nativamente en Rust (extremadamente rápido)
    let rule: Value = serde_json::from_str(rule_json)
        .map_err(|e| PyValueError::new_err(format!("Error parseando regla: {}", e)))?;
        
    let context: Value = serde_json::from_str(context_json)
        .map_err(|e| PyValueError::new_err(format!("Error parseando contexto: {}", e)))?;

    // 2. Ejecutar json-logic determinista
    let result = jsonlogic::apply(&rule, &context)
        .map_err(|e| PyValueError::new_err(format!("Error en json-logic: {}", e)))?;

    // 3. Extraer el resultado como f64 (flotante)
    match result {
        Value::Number(n) => n.as_f64().ok_or_else(|| PyValueError::new_err("El resultado numérico no pudo convertirse a f64")),
        Value::String(s) => s.parse::<f64>().map_err(|_| PyValueError::new_err("El resultado de la regla devolvió un string no numérico")),
        _ => Err(PyValueError::new_err("El resultado de la regla matemática debe ser numérico")),
    }
}

/// Evalúa una regla contra un lote masivo de transacciones.
/// Elimina el FFI overhead parseando la regla una sola vez y ejecutando el iterador en Rust.
#[pyfunction]
fn evaluate_batch(rule_json: &str, contexts_json: Vec<String>) -> PyResult<Vec<f64>> {
    let rule: Value = serde_json::from_str(rule_json)
        .map_err(|e| PyValueError::new_err(format!("Error parseando regla: {}", e)))?;
        
    let mut results = Vec::with_capacity(contexts_json.len());
    
    for ctx_str in contexts_json {
        let context: Value = match serde_json::from_str(&ctx_str) {
            Ok(c) => c,
            Err(_) => {
                results.push(0.0); // Omitimos errores de parseo individual en batch para no detener todo
                continue;
            }
        };

        match jsonlogic::apply(&rule, &context) {
            Ok(Value::Number(n)) => results.push(n.as_f64().unwrap_or(0.0)),
            Ok(Value::String(s)) => results.push(s.parse::<f64>().unwrap_or(0.0)),
            _ => results.push(0.0),
        }
    }
    
    Ok(results)
}

/// Modulo principal expuesto a Python
#[pymodule]
fn tempus_core(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(evaluate_fee, m)?)?;
    m.add_function(wrap_pyfunction!(evaluate_batch, m)?)?;
    Ok(())
}
