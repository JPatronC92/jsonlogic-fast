use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::error::RuleEngineResult;
use crate::guardrails::{evaluate_guardrails, Violation};
use crate::policy::RuleEnvelope;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AttemptLog {
    pub attempt: usize,
    pub raw_output: String,
    pub parsed_output: Option<Value>,
    pub feedback: Option<String>,
    pub valid: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OrchestrationResult {
    pub success: bool,
    pub attempts: Vec<AttemptLog>,
    pub output: Option<Value>,
    pub error_message: Option<String>,
}

pub fn render_violations(violations: &[Violation]) -> String {
    let mut msg = String::from("The generated output violated the following business rules:\n");
    for violation in violations {
        msg.push_str(&format!(
            "- [{}] Path '{}': {} (Actual: {:?})\n",
            violation.rule_id,
            violation.path.as_deref().unwrap_or("unknown"),
            violation.message,
            violation.actual
        ));
    }
    msg.push_str("Please correct these errors and output the revised valid JSON.");
    msg
}

pub fn orchestrate_reflection<F>(
    rules: &[RuleEnvelope],
    mut generate_fn: F,
    initial_messages: &[Value],
    max_retries: usize,
) -> RuleEngineResult<OrchestrationResult>
where
    F: FnMut(&[Value]) -> RuleEngineResult<String>,
{
    let mut messages = initial_messages.to_vec();
    let mut attempts = Vec::new();
    let mut previous_raw_output: Option<String> = None;

    for attempt in 0..=max_retries {
        // Call the LLM generator
        let raw_output = match generate_fn(&messages) {
            Ok(out) => out,
            Err(e) => {
                return Ok(OrchestrationResult {
                    success: false,
                    attempts,
                    output: None,
                    error_message: Some(format!("LLM generation failed: {}", e)),
                });
            }
        };

        // Check for identical output loop detection
        if let Some(ref prev) = previous_raw_output {
            if prev == &raw_output {
                return Ok(OrchestrationResult {
                    success: false,
                    attempts,
                    output: None,
                    error_message: Some("Infinite loop detected: LLM generated identical output as previous attempt.".to_string()),
                });
            }
        }
        previous_raw_output = Some(raw_output.clone());

        // Parse JSON
        let parsed_output: Option<Value> = serde_json::from_str(&raw_output).ok();

        if parsed_output.is_none() {
            let feedback =
                "The output is not a valid JSON. Please return a valid, parsable JSON object."
                    .to_string();
            attempts.push(AttemptLog {
                attempt,
                raw_output: raw_output.clone(),
                parsed_output: None,
                feedback: Some(feedback.clone()),
                valid: false,
            });

            // Append assistant response and error feedback to messages
            messages.push(json!({ "role": "assistant", "content": raw_output }));
            messages.push(json!({ "role": "user", "content": feedback }));
            continue;
        }

        let parsed_val = parsed_output.unwrap();

        // Evaluate guardrails
        let validation_report = match evaluate_guardrails(rules, &parsed_val) {
            Ok(report) => report,
            Err(e) => {
                return Ok(OrchestrationResult {
                    success: false,
                    attempts,
                    output: Some(parsed_val),
                    error_message: Some(format!("Guardrail evaluation execution error: {}", e)),
                });
            }
        };

        if validation_report.valid {
            // Success!
            attempts.push(AttemptLog {
                attempt,
                raw_output,
                parsed_output: Some(parsed_val.clone()),
                feedback: None,
                valid: true,
            });
            return Ok(OrchestrationResult {
                success: true,
                attempts,
                output: Some(parsed_val),
                error_message: None,
            });
        }

        // Check if any violation is retryable
        let any_retryable = validation_report.violations.iter().any(|v| v.retryable);
        if !any_retryable {
            attempts.push(AttemptLog {
                attempt,
                raw_output,
                parsed_output: Some(parsed_val.clone()),
                feedback: Some(
                    "Non-retryable violation encountered. Aborting orchestrator loop.".to_string(),
                ),
                valid: false,
            });
            return Ok(OrchestrationResult {
                success: false,
                attempts,
                output: Some(parsed_val),
                error_message: Some("Non-retryable violation encountered.".to_string()),
            });
        }

        // Render feedback
        let feedback = render_violations(&validation_report.violations);
        attempts.push(AttemptLog {
            attempt,
            raw_output: raw_output.clone(),
            parsed_output: Some(parsed_val.clone()),
            feedback: Some(feedback.clone()),
            valid: false,
        });

        // Append assistant response and error feedback to messages
        messages.push(json!({ "role": "assistant", "content": raw_output }));
        messages.push(json!({ "role": "user", "content": feedback }));
    }

    Ok(OrchestrationResult {
        success: false,
        attempts,
        output: None,
        error_message: Some("Max retries exceeded without passing all guardrails.".to_string()),
    })
}
