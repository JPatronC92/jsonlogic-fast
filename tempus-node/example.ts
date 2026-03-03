import { TempusClient } from "./src";

async function run() {
  const client = new TempusClient({
    apiKey: "YOUR_API_KEY_HERE",
    baseURL: "http://localhost:8000/api/v1",
  });

  console.log("🚀 Testing Tempus Node SDK...");

  try {
    // Test Single Calculation
    const calcRes = await client.calculate({
      scheme_urn: "urn:pricing:marketplace:mx",
      execution_date: new Date().toISOString(),
      transaction: {
        amount: 1000,
        currency: "MXN",
        country: "MX",
      },
    });

    console.log("\n✅ Single Calculation Result:");
    console.log(JSON.stringify(calcRes, null, 2));

    // Test Batch Simulation
    const transactions = Array.from({ length: 5 }, (_, i) => ({
      amount: 1500 + i * 100,
      currency: "MXN",
      country: "MX",
    }));

    const batchRes = await client.simulateBatch({
      scheme_urn: "urn:pricing:marketplace:mx",
      execution_date: new Date().toISOString(),
      transactions,
    });

    console.log("\n✅ Batch Simulation Result:");
    console.log(JSON.stringify(batchRes, null, 2));
  } catch (error: any) {
    console.error("❌ Error:", error.message);
  }
}

run();
