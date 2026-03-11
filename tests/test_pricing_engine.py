import uuid

import pytest

from src.domain.models import (PricingContextSchema, PricingRuleIdentity,
                               PricingRuleVersion)
from src.domain.services.pricing_engine import PricingEngine


@pytest.fixture
def active_rules():
    schema = PricingContextSchema(
        id=uuid.uuid4(),
        schema_json={
            "type": "object",
            "properties": {"amount": {"type": "number"}, "country": {"type": "string"}},
        },
    )

    rule_percent = PricingRuleIdentity(uuid=uuid.uuid4(), name="3.6% Base")
    v1_percent = PricingRuleVersion(
        id=uuid.uuid4(),
        rule_uuid=rule_percent.uuid,
        rule=rule_percent,
        context_schema=schema,
        logica_json={"*": [{"var": "amount"}, 0.036]},
        hash_firma="hash1",
    )

    rule_fixed = PricingRuleIdentity(uuid=uuid.uuid4(), name="$3.00 Fijo")
    v1_fixed = PricingRuleVersion(
        id=uuid.uuid4(),
        rule_uuid=rule_fixed.uuid,
        rule=rule_fixed,
        context_schema=schema,
        logica_json=3.00,
        hash_firma="hash2",
    )

    return [v1_percent, v1_fixed]


def test_pricing_engine_calculates_fees_correctly(active_rules):
    engine = PricingEngine()
    context = {"amount": 1000.00, "country": "MX", "currency": "MXN"}

    response = engine.calculate(context, active_rules)

    assert response.base_amount == 1000.00
    assert response.net_settlement == 961.00  # 1000 - 36 - 3
    assert response.total_fees == 39.00
    assert len(response.calculated_fees) == 2
    assert response.calculated_fees[0].amount == 36.00
    assert response.calculated_fees[1].amount == 3.00


def test_pricing_engine_fails_on_invalid_schema(active_rules):
    engine = PricingEngine()
    # Missing required fields according to schema (e.g. string instead of number)
    context = {"amount": "mil", "country": "MX"}

    with pytest.raises(ValueError, match="Payload malformado"):
        engine.calculate(context, active_rules)


def test_pricing_engine_simulate_batch(active_rules):
    engine = PricingEngine()
    transacciones = [
        {
            "amount": 1000.00,
            "country": "MX",
            "currency": "MXN",
        },  # Válida: fee 39 (36+3)
        {"amount": 500.00, "country": "MX", "currency": "MXN"},  # Válida: fee 21 (18+3)
        {"amount": "invalido", "country": "US"},  # Inválida
    ]

    response = engine.simulate_batch(transacciones, active_rules)

    assert response.total_processed_volume == 1500.00
    assert response.total_fees_collected == 60.00  # 39 + 21
    assert response.total_net_settlement == 1440.00  # 1500 - 60
    assert response.transactions_count == 2
    assert response.failed_transactions == 1
