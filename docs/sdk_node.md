# Node.js SDK

The Tempus Node.js SDK brings the deterministic precision of the Rust Engine to your frontend dashboards or Node.js microservices.

## Setup
The Node client natively provides TypeScript support.

```bash
npm install tempus-node
```

## Setup API Key Authentication

```typescript
import { TempusClient } from 'tempus-node';

const client = new TempusClient(
  'https://api.tempus.yourdomain.com/v1',
  'your_secret_api_key_here'
);
```

## Single Evaluated Fee
Use single evaluation endpoints to securely fetch dynamic pricing amounts before checkout, ensuring what a user sees in the UI accurately matches the final Rust execution.

```typescript
async function getCheckoutFee() {
  const result = await client.evaluateFee({
    scheme_urn: "urn:pricing:marketplace:mx",
    execution_date: new Date().toISOString().split('T')[0],
    transaction: {
      amount: 1540.50,
      total_volume: 100000
    }
  });

  console.log("Calculated Deterministic Fee:", result.fee);
}

getCheckoutFee();
```
