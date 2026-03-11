import time
from jsonschema import validate, Draft7Validator, ValidationError

def benchmark_jsonschema():
    schema = {
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

    instance = {"amount": 1000.00, "country": "MX", "currency": "MXN", "user_id": "user_123", "is_prime": True}

    iterations = 10000

    print(f"Benchmarking {iterations} iterations...")

    # Baseline: validate()
    start = time.time()
    for _ in range(iterations):
        validate(instance=instance, schema=schema)
    end = time.time()
    baseline_time = end - start
    print(f"jsonschema.validate(): {baseline_time:.4f}s ({(baseline_time/iterations)*1000:.4f}ms per call)")

    # Optimized: Cached Validator
    validator = Draft7Validator(schema)
    start = time.time()
    for _ in range(iterations):
        validator.validate(instance)
    end = time.time()
    optimized_time = end - start
    print(f"Draft7Validator.validate(): {optimized_time:.4f}s ({(optimized_time/iterations)*1000:.4f}ms per call)")

    improvement = (baseline_time - optimized_time) / baseline_time * 100
    print(f"Improvement: {improvement:.2f}%")

if __name__ == "__main__":
    try:
        benchmark_jsonschema()
    except ImportError as e:
        print(f"ImportError: {e}")
    except Exception as e:
        print(f"Error: {e}")
