import pytest
from jsonlogic_fast import (
    RuleEnvelope,
    Violation,
    GuardrailReport,
    Router,
    AttemptLog,
    OrchestrationResult,
    evaluate_guardrails,
    orchestrate_reflection,
)


def test_rule_envelope_creation():
    rule = RuleEnvelope(
        id="score_limit",
        assert_val={">": [{"var": "score"}, 500]},
        message="Score is below limit",
        path="$.score",
        severity="error",
        retryable=True,
    )
    assert rule.id == "score_limit"
    assert rule.assert_val == {">": [{"var": "score"}, 500]}
    assert rule.message == "Score is below limit"
    assert rule.path == "$.score"
    assert rule.severity == "error"
    assert rule.retryable is True


def test_evaluate_guardrails_pass_and_fail():
    rules = [
        RuleEnvelope(
            id="temp_max",
            assert_val={"<": [{"var": "temp"}, 40]},
            message="Temperature too high",
            path="$.temp",
            severity="error",
            retryable=True,
        ),
        RuleEnvelope(
            id="status_ok",
            assert_val={"==": [{"var": "status"}, "success"]},
            message="Status must be success",
            path="$.status",
            severity="warning",
            retryable=False,
        ),
    ]

    # Case 1: Both pass
    report = evaluate_guardrails(rules, {"temp": 30, "status": "success"})
    assert report.valid is True
    assert len(report.violations) == 0

    # Case 2: One fails
    report2 = evaluate_guardrails(rules, {"temp": 45, "status": "success"})
    assert report2.valid is False
    assert len(report2.violations) == 1
    v = report2.violations[0]
    assert v.rule_id == "temp_max"
    assert v.path == "$.temp"
    assert v.message == "Temperature too high"
    assert v.actual == 45
    assert v.severity == "error"
    assert v.retryable is True

    # Case 3: Both fail
    report3 = evaluate_guardrails(rules, {"temp": 45, "status": "failed"})
    assert report3.valid is False
    assert len(report3.violations) == 2


def test_router_deterministic():
    rule = {
        "if": [
            {"var": "request.contains_pii"},
            "anonymize_pipeline",
            {
                "if": [
                    {"var": "environment.local_model_available"},
                    "local_llm",
                    "cloud_llm",
                ]
            },
        ]
    }
    router = Router(rule)

    decision = router.route(
        {
            "request": {"contains_pii": True},
            "environment": {"local_model_available": True},
        }
    )
    assert decision == "anonymize_pipeline"

    decision2 = router.route(
        {
            "request": {"contains_pii": False},
            "environment": {"local_model_available": True},
        }
    )
    assert decision2 == "local_llm"

    decision3 = router.route(
        {
            "request": {"contains_pii": False},
            "environment": {"local_model_available": False},
        }
    )
    assert decision3 == "cloud_llm"


def test_orchestrate_reflection_immediate_success():
    rules = [
        RuleEnvelope(
            id="score_limit",
            assert_val={">": [{"var": "score"}, 500]},
            message="Score too low",
            path="$.score",
        )
    ]

    def mock_generator(messages):
        return '{"score": 700}'

    result = orchestrate_reflection(
        rules, mock_generator, [{"role": "user", "content": "hello"}], 3
    )
    assert result.success is True
    assert len(result.attempts) == 1
    assert result.output == {"score": 700}
    assert result.attempts[0].valid is True
    assert result.error_message is None


def test_orchestrate_reflection_retry_success():
    rules = [
        RuleEnvelope(
            id="score_limit",
            assert_val={">": [{"var": "score"}, 500]},
            message="Score too low",
            path="$.score",
        )
    ]

    calls = 0

    def mock_generator(messages):
        nonlocal calls
        calls += 1
        if calls == 1:
            return '{"score": 300}'
        else:
            # Check that we received feedback in user message
            last_msg = messages[-1]
            assert last_msg["role"] == "user"
            assert "score_limit" in last_msg["content"]
            return '{"score": 600}'

    result = orchestrate_reflection(rules, mock_generator, [], 3)
    assert result.success is True
    assert len(result.attempts) == 2
    assert result.output == {"score": 600}
    assert result.attempts[0].valid is False
    assert result.attempts[0].feedback is not None
    assert result.attempts[1].valid is True


def test_orchestrate_reflection_abort_non_retryable():
    rules = [
        RuleEnvelope(
            id="score_critical",
            assert_val={">": [{"var": "score"}, 500]},
            message="Score too low, critical!",
            path="$.score",
            retryable=False,
        )
    ]

    def mock_generator(messages):
        return '{"score": 300}'

    result = orchestrate_reflection(rules, mock_generator, [], 3)
    assert result.success is False
    assert len(result.attempts) == 1
    assert result.attempts[0].valid is False
    assert "Non-retryable" in result.attempts[0].feedback
    assert result.error_message == "Non-retryable violation encountered."


def test_orchestrate_reflection_abort_mixed_retryable():
    rules = [
        RuleEnvelope(
            id="score_retryable",
            assert_val={">": [{"var": "score"}, 500]},
            message="Score too low, but retryable",
            path="$.score",
            retryable=True,
        ),
        RuleEnvelope(
            id="score_critical",
            assert_val={">": [{"var": "score"}, 100]},
            message="Score below 100, critical!",
            path="$.score",
            retryable=False,
        )
    ]

    def mock_generator(messages):
        return '{"score": 50}'

    result = orchestrate_reflection(rules, mock_generator, [], 3)
    assert result.success is False
    assert len(result.attempts) == 1
    assert result.attempts[0].valid is False
    assert "Non-retryable" in result.attempts[0].feedback
    assert result.error_message == "Non-retryable violation encountered."


def test_orchestrate_reflection_loop_detection():
    rules = [
        RuleEnvelope(
            id="score_limit",
            assert_val={">": [{"var": "score"}, 500]},
            message="Score too low",
            path="$.score",
        )
    ]

    def mock_generator(messages):
        return '{"score": 300}'

    result = orchestrate_reflection(rules, mock_generator, [], 3)
    assert result.success is False
    assert len(result.attempts) == 1
    assert "Infinite loop" in result.error_message


def test_compile_dsl():
    from jsonlogic_fast import compile_dsl

    dsl = """
        rule approve_credit:
          when score > 700
          then "approve"
          else "review"
    """
    compiled = compile_dsl(dsl)
    assert compiled == {
        "if": [
            {">": [{"var": "score"}, 700.0]},
            "approve",
            "review"
        ]
    }


def test_compile_dsl_with_array_literal():
    from jsonlogic_fast import compile_dsl

    dsl = """
        rule category_check:
          when category in ["premium", "gold"]
          then "allowed"
          else "denied"
    """
    compiled = compile_dsl(dsl)
    assert compiled == {
        "if": [
            {"in": [{"var": "category"}, ["premium", "gold"]]},
            "allowed",
            "denied"
        ]
    }
