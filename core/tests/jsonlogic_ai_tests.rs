use jsonlogic_fast::error::RuleEngineError;
use jsonlogic_fast::guardrails::evaluate_guardrails;
use jsonlogic_fast::orchestrator::{orchestrate_reflection, render_violations};
use jsonlogic_fast::policy::{RuleEnvelope, Severity};
use jsonlogic_fast::router::Router;
use serde_json::{json, Value};

#[test]
fn test_policy_serialization() {
    let raw_policy = r#"{
        "id": "risk_assessment",
        "assert": {">": [{"var": "score"}, 700]},
        "message": "Risk score is too low",
        "path": "$.score",
        "severity": "error",
        "retryable": true
    }"#;

    let envelope: RuleEnvelope = serde_json::from_str(raw_policy).unwrap();
    assert_eq!(envelope.id, "risk_assessment");
    assert_eq!(envelope.severity, Severity::Error);
    assert!(envelope.retryable);
    assert_eq!(envelope.path, Some("$.score".to_string()));
}

#[test]
fn test_evaluate_guardrails_pass_and_fail() {
    let rules = vec![
        RuleEnvelope {
            id: "temp_max".to_string(),
            assert: json!({"<": [{"var": "temperature"}, 40]}),
            message: "Temperature must be less than 40.".to_string(),
            path: Some("$.temperature".to_string()),
            severity: Severity::Error,
            retryable: true,
        },
        RuleEnvelope {
            id: "status_check".to_string(),
            assert: json!({"==": [{"var": "status"}, "success"]}),
            message: "Status must be success.".to_string(),
            path: Some("$.status".to_string()),
            severity: Severity::Warning,
            retryable: false,
        },
    ];

    // Case 1: Both pass
    let input_pass = json!({
        "temperature": 25,
        "status": "success"
    });
    let report_pass = evaluate_guardrails(&rules, &input_pass).unwrap();
    assert!(report_pass.valid);
    assert_eq!(report_pass.violations.len(), 0);

    // Case 2: One fails
    let input_fail = json!({
        "temperature": 45,
        "status": "success"
    });
    let report_fail = evaluate_guardrails(&rules, &input_fail).unwrap();
    assert!(!report_fail.valid);
    assert_eq!(report_fail.violations.len(), 1);
    assert_eq!(report_fail.violations[0].rule_id, "temp_max");
    assert_eq!(report_fail.violations[0].actual, Some(json!(45)));
    assert_eq!(report_fail.violations[0].severity, Severity::Error);

    // Case 3: Both fail
    let input_fail_both = json!({
        "temperature": 45,
        "status": "failed"
    });
    let report_fail_both = evaluate_guardrails(&rules, &input_fail_both).unwrap();
    assert!(!report_fail_both.valid);
    assert_eq!(report_fail_both.violations.len(), 2);

    let feedback = render_violations(&report_fail_both.violations);
    assert!(feedback.contains("temp_max"));
    assert!(feedback.contains("status_check"));
}

#[test]
fn test_router_deterministic() {
    let rule = json!({
        "if": [
            {"var": "request.contains_pii"},
            "anonymize_pipeline",
            {
                "if": [
                    {"var": "environment.local_model_available"},
                    "local_llm",
                    "cloud_llm"
                ]
            }
        ]
    });

    let router = Router::new(rule);

    // Routing context 1: contains PII
    let context1 = json!({
        "request": {
            "contains_pii": true
        },
        "environment": {
            "local_model_available": true
        }
    });
    let decision1 = router.route(&context1).unwrap();
    assert_eq!(decision1, json!("anonymize_pipeline"));

    // Routing context 2: no PII, local model available
    let context2 = json!({
        "request": {
            "contains_pii": false
        },
        "environment": {
            "local_model_available": true
        }
    });
    let decision2 = router.route(&context2).unwrap();
    assert_eq!(decision2, json!("local_llm"));

    // Routing context 3: no PII, local model not available
    let context3 = json!({
        "request": {
            "contains_pii": false
        },
        "environment": {
            "local_model_available": false
        }
    });
    let decision3 = router.route(&context3).unwrap();
    assert_eq!(decision3, json!("cloud_llm"));
}

#[test]
fn test_orchestration_loop_success_immediate() {
    let rules = vec![RuleEnvelope {
        id: "score_ok".to_string(),
        assert: json!({">": [{"var": "score"}, 50]}),
        message: "Score too low.".to_string(),
        path: Some("$.score".to_string()),
        severity: Severity::Error,
        retryable: true,
    }];

    let mock_generator = |_messages: &[Value]| -> Result<String, RuleEngineError> {
        Ok(r#"{"score": 75}"#.to_string())
    };

    let result = orchestrate_reflection(&rules, mock_generator, &[], 3).unwrap();
    assert!(result.success);
    assert_eq!(result.attempts.len(), 1);
    assert_eq!(result.output, Some(json!({"score": 75})));
    assert!(result.attempts[0].valid);
}

#[test]
fn test_orchestration_loop_success_after_retry() {
    let rules = vec![RuleEnvelope {
        id: "score_ok".to_string(),
        assert: json!({">": [{"var": "score"}, 50]}),
        message: "Score too low.".to_string(),
        path: Some("$.score".to_string()),
        severity: Severity::Error,
        retryable: true,
    }];

    let mut call_count = 0;
    let mock_generator = move |messages: &[Value]| -> Result<String, RuleEngineError> {
        call_count += 1;
        if call_count == 1 {
            // First time, return failing JSON
            Ok(r#"{"score": 30}"#.to_string())
        } else {
            // Check that we received feedback in the messages
            assert!(messages.len() >= 2);
            let feedback = messages
                .last()
                .unwrap()
                .get("content")
                .unwrap()
                .as_str()
                .unwrap();
            assert!(feedback.contains("score_ok"));
            Ok(r#"{"score": 80}"#.to_string())
        }
    };

    let result = orchestrate_reflection(&rules, mock_generator, &[], 3).unwrap();
    assert!(result.success);
    assert_eq!(result.attempts.len(), 2);
    assert_eq!(result.output, Some(json!({"score": 80})));
    assert!(!result.attempts[0].valid);
    assert!(result.attempts[1].valid);
}

#[test]
fn test_orchestration_loop_non_retryable_abort() {
    let rules = vec![RuleEnvelope {
        id: "score_critical".to_string(),
        assert: json!({">": [{"var": "score"}, 80]}),
        message: "Score too low and not retryable.".to_string(),
        path: Some("$.score".to_string()),
        severity: Severity::Error,
        retryable: false,
    }];

    let mock_generator = |_messages: &[Value]| -> Result<String, RuleEngineError> {
        Ok(r#"{"score": 50}"#.to_string())
    };

    let result = orchestrate_reflection(&rules, mock_generator, &[], 3).unwrap();
    assert!(!result.success);
    assert_eq!(result.attempts.len(), 1);
    assert!(!result.attempts[0].valid);
    assert!(result.attempts[0]
        .feedback
        .as_ref()
        .unwrap()
        .contains("Non-retryable violation"));
}

#[test]
fn test_orchestration_loop_detection() {
    let rules = vec![RuleEnvelope {
        id: "score_ok".to_string(),
        assert: json!({">": [{"var": "score"}, 50]}),
        message: "Score too low.".to_string(),
        path: Some("$.score".to_string()),
        severity: Severity::Error,
        retryable: true,
    }];

    // Generator always returns the same invalid score
    let mock_generator = |_messages: &[Value]| -> Result<String, RuleEngineError> {
        Ok(r#"{"score": 30}"#.to_string())
    };

    let result = orchestrate_reflection(&rules, mock_generator, &[], 3).unwrap();
    assert!(!result.success);
    // Should stop immediately at attempt 1 (index 1) because the second call generates the same raw output
    assert_eq!(result.attempts.len(), 1);
    assert!(result
        .error_message
        .as_ref()
        .unwrap()
        .contains("Infinite loop detected"));
}

#[test]
fn test_dsl_compiler_success() {
    let dsl = r#"
        rule approve_credit:
          when score > 700
          then "approve"
          else "review"
    "#;

    let compiled = jsonlogic_fast::compiler_dsl::compile_dsl(dsl).unwrap();
    assert_eq!(
        compiled,
        json!({
            "if": [
                {">": [{"var": "score"}, 700.0]},
                "approve",
                "review"
            ]
        })
    );
}
