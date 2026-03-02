# Python SDK

The Tempus Python SDK is a lightweight `httpx`-based async client that allows seamless integration with the B2B API to trigger single or batch billing calculations.

## Installation
The SDK relies on `pydantic` schemas for typing and validation. You can install it natively or add the source to your `pyproject.toml`.

## Initialization
```python
from tempus.client import TempusClient

# The client uses 'http://localhost:8000/api/v1' by default
# Pass your tenant API Key to the constructor
client = TempusClient(
    base_url="https://api.tempus.yourdomain.com/v1",
    api_key="your_secret_api_key_here"
)
```

## Batch Simulations
Simulating 1,000,000 transactions at once using the Time-Travel Engine:
```python
import asyncio
from typing import Dict, Any

async def run_billing_cycle():
    # Construct an array of transactions
    transactions = [
        {"amount": 500, "method": "CREDIT_CARD"},
        {"amount": 1000, "method": "CASH"}
    ] * 500000  # 1 Million txs

    # The evaluation happens in Rust (tempus_core) ensuring strict determinism
    # The 'execution_date' ensures overlapping or future tier pricing rules don't affect past billing reconciliations
    results = await client.evaluate_batch(
        scheme_urn="urn:pricing:marketplace:mx",
        execution_date="2026-03-01",
        transactions=transactions
    )

    print(f"Processed 1M transactions in: {results.time_elapsed_ms} ms")
    print(results.evaluated_fees[:5])

if __name__ == "__main__":
    asyncio.run(run_billing_cycle())
```
