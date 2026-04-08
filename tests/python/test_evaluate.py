"""End-to-end tests for the jsonlogic-fast Python bindings.

These tests exercise every public function exposed by the ``jsonlogic_fast``
module, including happy paths, edge cases, and expected error conditions.
"""

import json
import pytest
import jsonlogic_fast


# ---------------------------------------------------------------------------
# Fixtures
# ---------------------------------------------------------------------------

SIMPLE_RULE = json.dumps({">": [{"var": "score"}, 700]})
CONDITIONAL_RULE = json.dumps(
    {"if": [{">": [{"var": "score"}, 700]}, "approve", "review"]}
)
NUMERIC_RULE = json.dumps({"*": [{"var": "amount"}, 0.029]})
ALWAYS_TRUE_RULE = json.dumps({"==": [1, 1]})
INVALID_JSON = "not valid json {{"


def _ctx(**kwargs) -> str:
    return json.dumps(kwargs)


# ---------------------------------------------------------------------------
# evaluate
# ---------------------------------------------------------------------------

class TestEvaluate:
    def test_returns_python_bool(self):
        result = jsonlogic_fast.evaluate(SIMPLE_RULE, _ctx(score=800))
        assert result is True

    def test_returns_python_string(self):
        result = jsonlogic_fast.evaluate(CONDITIONAL_RULE, _ctx(score=742))
        assert result == "approve"

    def test_returns_python_string_low_score(self):
        result = jsonlogic_fast.evaluate(CONDITIONAL_RULE, _ctx(score=500))
        assert result == "review"

    def test_invalid_rule_json_raises(self):
        with pytest.raises(ValueError):
            jsonlogic_fast.evaluate(INVALID_JSON, _ctx(score=1))

    def test_invalid_context_json_raises(self):
        with pytest.raises(ValueError):
            jsonlogic_fast.evaluate(SIMPLE_RULE, INVALID_JSON)


# ---------------------------------------------------------------------------
# evaluate_rule (alias)
# ---------------------------------------------------------------------------

class TestEvaluateRule:
    def test_alias_matches_evaluate(self):
        a = jsonlogic_fast.evaluate(CONDITIONAL_RULE, _ctx(score=742))
        b = jsonlogic_fast.evaluate_rule(CONDITIONAL_RULE, _ctx(score=742))
        assert a == b


# ---------------------------------------------------------------------------
# evaluate_json
# ---------------------------------------------------------------------------

class TestEvaluateJson:
    def test_returns_json_string(self):
        result = jsonlogic_fast.evaluate_json(CONDITIONAL_RULE, _ctx(score=742))
        assert json.loads(result) == "approve"

    def test_numeric_result(self):
        result = jsonlogic_fast.evaluate_json(NUMERIC_RULE, _ctx(amount=100))
        value = json.loads(result)
        assert pytest.approx(value, rel=1e-6) == 2.9

    def test_invalid_rule_raises(self):
        with pytest.raises(ValueError):
            jsonlogic_fast.evaluate_json(INVALID_JSON, _ctx(x=1))


# ---------------------------------------------------------------------------
# evaluate_numeric
# ---------------------------------------------------------------------------

class TestEvaluateNumeric:
    def test_basic_multiplication(self):
        result = jsonlogic_fast.evaluate_numeric(NUMERIC_RULE, _ctx(amount=1000))
        assert pytest.approx(result, rel=1e-6) == 29.0

    def test_boolean_true_coerces_to_one(self):
        result = jsonlogic_fast.evaluate_numeric(ALWAYS_TRUE_RULE, _ctx())
        assert result == 1.0

    def test_string_result_raises(self):
        with pytest.raises(ValueError):
            jsonlogic_fast.evaluate_numeric(CONDITIONAL_RULE, _ctx(score=800))


# ---------------------------------------------------------------------------
# evaluate_batch
# ---------------------------------------------------------------------------

class TestEvaluateBatch:
    def test_batch_returns_list(self):
        contexts = [_ctx(score=800), _ctx(score=500), _ctx(score=742)]
        results = jsonlogic_fast.evaluate_batch(CONDITIONAL_RULE, contexts)
        assert results == ["approve", "review", "approve"]

    def test_empty_batch(self):
        results = jsonlogic_fast.evaluate_batch(CONDITIONAL_RULE, [])
        assert results == []

    def test_single_item_batch(self):
        results = jsonlogic_fast.evaluate_batch(CONDITIONAL_RULE, [_ctx(score=100)])
        assert results == ["review"]


# ---------------------------------------------------------------------------
# evaluate_batch_json
# ---------------------------------------------------------------------------

class TestEvaluateBatchJson:
    def test_returns_json_array(self):
        contexts = [_ctx(score=800), _ctx(score=500)]
        result = jsonlogic_fast.evaluate_batch_json(CONDITIONAL_RULE, contexts)
        parsed = json.loads(result)
        assert parsed == ["approve", "review"]

    def test_empty_batch(self):
        result = jsonlogic_fast.evaluate_batch_json(CONDITIONAL_RULE, [])
        assert result == "[]"

    def test_invalid_rule_raises(self):
        with pytest.raises(ValueError):
            jsonlogic_fast.evaluate_batch_json(INVALID_JSON, [_ctx(score=1)])

    def test_invalid_context_returns_null(self):
        result = jsonlogic_fast.evaluate_batch_json(CONDITIONAL_RULE, [INVALID_JSON])
        parsed = json.loads(result)
        assert parsed == [None]


# ---------------------------------------------------------------------------
# evaluate_batch_detailed
# ---------------------------------------------------------------------------

class TestEvaluateBatchDetailed:
    def test_success_has_result_no_error(self):
        contexts = [_ctx(score=800)]
        results = jsonlogic_fast.evaluate_batch_detailed(CONDITIONAL_RULE, contexts)
        assert len(results) == 1
        item = results[0]
        assert item["result"] == "approve"
        assert item["error"] is None

    def test_invalid_context_has_error(self):
        contexts = [INVALID_JSON]
        results = jsonlogic_fast.evaluate_batch_detailed(CONDITIONAL_RULE, contexts)
        assert len(results) == 1
        item = results[0]
        assert item["error"] is not None
        assert len(item["error"]) > 0


# ---------------------------------------------------------------------------
# evaluate_batch_numeric
# ---------------------------------------------------------------------------

class TestEvaluateBatchNumeric:
    def test_batch_numeric_results(self):
        contexts = [_ctx(amount=100), _ctx(amount=200), _ctx(amount=0)]
        results = jsonlogic_fast.evaluate_batch_numeric(NUMERIC_RULE, contexts)
        assert len(results) == 3
        assert pytest.approx(results[0], rel=1e-6) == 2.9
        assert pytest.approx(results[1], rel=1e-6) == 5.8
        assert results[2] == 0.0


# ---------------------------------------------------------------------------
# evaluate_batch_numeric_detailed
# ---------------------------------------------------------------------------

class TestEvaluateBatchNumericDetailed:
    def test_returns_tuple_of_values_and_errors(self):
        contexts = [_ctx(amount=100), _ctx(amount=200)]
        values, errors = jsonlogic_fast.evaluate_batch_numeric_detailed(
            NUMERIC_RULE, contexts
        )
        assert len(values) == 2
        assert len(errors) == 2
        assert pytest.approx(values[0], rel=1e-6) == 2.9
        assert errors[0] == ""  # no error

    def test_invalid_context_returns_zero_with_error(self):
        contexts = [INVALID_JSON]
        values, errors = jsonlogic_fast.evaluate_batch_numeric_detailed(
            NUMERIC_RULE, contexts
        )
        assert values[0] == 0.0
        assert len(errors[0]) > 0


# ---------------------------------------------------------------------------
# validate_rule
# ---------------------------------------------------------------------------

class TestValidateRule:
    def test_valid_rule(self):
        assert jsonlogic_fast.validate_rule(SIMPLE_RULE) is True

    def test_always_true_rule(self):
        assert jsonlogic_fast.validate_rule(ALWAYS_TRUE_RULE) is True

    def test_invalid_json_raises(self):
        with pytest.raises(ValueError):
            jsonlogic_fast.validate_rule(INVALID_JSON)


# ---------------------------------------------------------------------------
# get_core_info
# ---------------------------------------------------------------------------

class TestGetCoreInfo:
    def test_returns_valid_json(self):
        info = jsonlogic_fast.get_core_info()
        parsed = json.loads(info)
        assert isinstance(parsed, dict)

    def test_contains_version(self):
        info = json.loads(jsonlogic_fast.get_core_info())
        assert "version" in info

    def test_contains_parallelism(self):
        info = json.loads(jsonlogic_fast.get_core_info())
        assert "parallelism" in info


# ---------------------------------------------------------------------------
# Determinism
# ---------------------------------------------------------------------------

class TestDeterminism:
    """Core promise: same rule + same context = same result, always."""

    def test_repeated_evaluation_is_identical(self):
        ctx = _ctx(score=742, country="MX")
        results = [jsonlogic_fast.evaluate_json(CONDITIONAL_RULE, ctx) for _ in range(100)]
        assert len(set(results)) == 1

    def test_batch_matches_individual(self):
        contexts = [_ctx(score=800), _ctx(score=500), _ctx(score=742)]
        batch = jsonlogic_fast.evaluate_batch(CONDITIONAL_RULE, contexts)
        individual = [jsonlogic_fast.evaluate(CONDITIONAL_RULE, c) for c in contexts]
        assert batch == individual
