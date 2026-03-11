import statistics
import time
import uuid

from src.domain.models import (PricingContextSchema, PricingRuleIdentity,
                               PricingRuleVersion)
from src.domain.services.pricing_engine import PricingEngine


def create_active_rules(num_rules=100):
    schema = PricingContextSchema(
        id=uuid.uuid4(),
        schema_json={
            "type": "object",
            "properties": {"amount": {"type": "number"}, "country": {"type": "string"}},
        },
    )

    rules = []
    for i in range(num_rules):
        rule = PricingRuleIdentity(uuid=uuid.uuid4(), name=f"Rule {i}")
        v1 = PricingRuleVersion(
            id=uuid.uuid4(),
            rule_uuid=rule.uuid,
            rule=rule,
            context_schema=schema,
            logica_json={"*": [{"var": "amount"}, 0.01]},
            hash_firma=f"hash{i}",
        )
        rules.append(v1)

    return rules


def run_benchmark():
    engine = PricingEngine()
    rules = create_active_rules(100)
    context = {"amount": 1000.00, "country": "MX", "currency": "MXN"}

    # Warmup
    for _ in range(10):
        engine.calculate(context, rules)

    iterations = 1000
    times = []

    for _ in range(iterations):
        start = time.perf_counter()
        engine.calculate(context, rules)
        end = time.perf_counter()
        times.append(end - start)

    avg_time = statistics.mean(times)
    median_time = statistics.median(times)

    print(f"Benchmark Results (100 rules, {iterations} iterations):")
    print(f"Average time per calculate call: {avg_time * 1000:.3f} ms")
    print(f"Median time per calculate call:  {median_time * 1000:.3f} ms")


if __name__ == "__main__":
    run_benchmark()
