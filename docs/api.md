# REST API Reference

The Tempus Engine REST API allows you to simulate and perform evaluations for your pricing rules. Base URL is typically `http://api.tempus.local/v1`.

## Authentication
Every request must include the `X-API-Key` header with your active tenant API key.

```bash
curl -X POST http://localhost:8000/api/v1/billing/evaluate \
  -H "X-API-Key: your_secret_key" \
  -H "Content-Type: application/json"
```

## Endpoints

### 1. Single Transaction Evaluation
`POST /api/v1/billing/evaluate`

Evaluates a single transaction against an active pricing scheme.

**Request Body:**
```json
{
  "scheme_urn": "urn:pricing:marketplace:mx",
  "execution_date": "2026-03-01",
  "transaction": {
    "amount": 1000,
    "method": "CREDIT_CARD"
  }
}
```

### 2. High-Speed Batch Evaluation
`POST /api/v1/billing/batch`

Process an array of thousands of transactions directly within the Rust Core engine.

**Request Body:**
```json
{
  "scheme_urn": "urn:pricing:marketplace:mx",
  "execution_date": "2026-03-01",
  "transactions": [
    { "amount": 1000 },
    { "amount": 500 },
    { "amount": 25000 }
  ]
}
```

**Returns:**
```json
{
  "evaluated_fees": [30.0, 15.0, 750.0],
  "time_elapsed_ms": 1.2
}
```
