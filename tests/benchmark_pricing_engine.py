import time
import uuid
import json
from src.domain.services.pricing_engine import PricingEngine
from src.domain.models import (
    PricingRuleVersion,
    PricingRuleIdentity,
    PricingContextSchema,
)

def benchmark():
    schema_json = {
        "type": "object",
        "properties": {
            "amount": {"type": "number"},
            "country": {"type": "string"},
            "currency": {"type": "string"},
            "user_id": {"type": "string"},
            "is_prime": {"type": "boolean"},
        },
        "required": ["amount", "country", "currency"]
    }

    schema = PricingContextSchema(
        id=uuid.uuid4(),
        schema_json=schema_json,
    )

    reglas_activas = []
    for i in range(50):
        rule = PricingRuleIdentity(uuid=uuid.uuid4(), name=f"Rule {i}")
        rv = PricingRuleVersion(
            id=uuid.uuid4(),
            rule_uuid=rule.uuid,
            rule=rule,
            context_schema=schema,
            logica_json={"*": [{"var": "amount"}, 0.001 * (i + 1)]},
            hash_firma=f"hash{i}",
        )
        reglas_activas.append(rv)

    engine = PricingEngine()
    context = {"amount": 1000.00, "country": "MX", "currency": "MXN", "user_id": "user_123", "is_prime": True}

    # Warm up
    for _ in range(5):
        engine.calculate(context, reglas_activas)

    print("Starting benchmark...")
    start = time.time()
    iterations = 200
    for _ in range(iterations):
        engine.calculate(context, reglas_activas)
    end = time.time()

    total_time = end - start
    avg_time = total_time / iterations
    print(f"Total time for {iterations} iterations with {len(reglas_activas)} rules: {total_time:.4f}s")
    print(f"Average time per calculate call: {avg_time * 1000:.4f}ms")

if __name__ == "__main__":
    benchmark()
