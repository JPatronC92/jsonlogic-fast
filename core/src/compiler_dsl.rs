use serde_json::{json, Value};

/// Parses an expression, which can be a string literal, boolean literal, numeric literal, or a variable reference.
fn parse_expression(expr: &str) -> Result<Value, String> {
    let trimmed = expr.trim();
    if trimmed.is_empty() {
        return Err("Empty expression".to_string());
    }

    // 1. String literal
    if (trimmed.starts_with('"') && trimmed.ends_with('"'))
        || (trimmed.starts_with('\'') && trimmed.ends_with('\''))
    {
        return Ok(json!(trimmed[1..trimmed.len() - 1].to_string()));
    }

    // 2. Boolean literal
    if trimmed == "true" {
        return Ok(json!(true));
    }
    if trimmed == "false" {
        return Ok(json!(false));
    }

    // 3. Numeric literal
    if let Ok(val) = trimmed.parse::<f64>() {
        return Ok(json!(val));
    }

    // 4. Variable reference
    Ok(json!({ "var": trimmed }))
}

/// Parses a comparison condition (e.g., "score > 700" or "status == 'success'").
fn parse_condition(cond: &str) -> Result<Value, String> {
    let trimmed = cond.trim();
    let operators = ["==", "!=", ">=", "<=", ">", "<", " in "];
    for op in &operators {
        if let Some(pos) = trimmed.find(op) {
            let left = trimmed[..pos].trim();
            let right = trimmed[pos + op.len()..].trim();

            let parsed_op = op.trim();
            let parsed_left = parse_expression(left)?;
            let parsed_right = parse_expression(right)?;

            return Ok(json!({
                parsed_op: [parsed_left, parsed_right]
            }));
        }
    }
    Err(format!(
        "Could not parse comparison condition: '{}'",
        trimmed
    ))
}

/// Compiles a single DSL rule string into its corresponding JSON-Logic representation.
pub fn compile_dsl(dsl_str: &str) -> Result<Value, String> {
    let lines: Vec<&str> = dsl_str
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .collect();

    if lines.is_empty() {
        return Err("Empty DSL input".to_string());
    }

    let mut _rule_name = "anonymous".to_string();
    let mut when_condition = None;
    let mut then_expr = None;
    let mut else_expr = None;

    let mut i = 0;
    while i < lines.len() {
        let line = lines[i];
        if let Some(rest) = line.strip_prefix("rule ") {
            _rule_name = rest.trim_end_matches(':').trim().to_string();
            i += 1;
        } else if let Some(rest) = line.strip_prefix("when ") {
            when_condition = Some(rest.trim().to_string());
            i += 1;
        } else if let Some(rest) = line.strip_prefix("then ") {
            then_expr = Some(rest.trim().to_string());
            i += 1;
        } else if let Some(rest) = line.strip_prefix("else ") {
            else_expr = Some(rest.trim().to_string());
            i += 1;
        } else {
            return Err(format!("Unexpected token or line: '{}'", line));
        }
    }

    let when_cond_str = when_condition.ok_or_else(|| "Missing 'when' condition".to_string())?;
    let then_expr_str = then_expr.ok_or_else(|| "Missing 'then' expression".to_string())?;
    let else_expr_str = else_expr.ok_or_else(|| "Missing 'else' expression".to_string())?;

    let assert_val = parse_condition(&when_cond_str)?;
    let parsed_then = parse_expression(&then_expr_str)?;
    let parsed_else = parse_expression(&else_expr_str)?;

    Ok(json!({
        "if": [
            assert_val,
            parsed_then,
            parsed_else
        ]
    }))
}
