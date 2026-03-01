import { NextResponse } from "next/server";
import { TempusClient } from "tempus-node";

export async function POST(req: Request) {
  try {
    const body = await req.json();
    const client = new TempusClient({
      apiKey: "test-api-key",
      baseURL: "http://localhost:8001/api/v1",
    });

    const response = await client.simulateBatch(body);
    return NextResponse.json(response);
  } catch (error: any) {
    return NextResponse.json({ error: error.message }, { status: 500 });
  }
}
