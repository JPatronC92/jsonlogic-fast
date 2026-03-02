# Tempus Documentation

Welcome to the **Tempus** developer documentation.

Tempus is a high-performance **Deterministic Billing and Commission Engine** written in Rust and wrapped in a FastAPI service. It is designed to evaluate massive batches of financial transactions against complex pricing rules with **absolute certainty, zero floating-point drift, and sub-millisecond precision.**

## Core Concepts

### Time-Travel Audit
Every pricing rule in Tempus carries a strict start and end date using PostgreSQL's `DATERANGE`. This ensures that given a specific date in history, there is exactly one mathematical rule evaluated. 

### Determinism via JSON-Logic
Instead of hard-coded `if/else` commission splits, Tempus stores pricing logic as `json-logic` blobs. This is parsed natively by our Rust core `tempus_core` yielding up to **1.3M TPS**.

## Getting Started

To integrate Tempus into your platform, you can use our REST API or our official SDKs:

- [REST API](api.md)
- [Python SDK](sdk_python.md)
- [Node.js SDK](sdk_node.md)

## Security
All external interactions with Tempus are authenticated via **API Keys**. You must pass the API Key in the `X-API-Key` header for B2B API integrations, or configure it when initializing the SDK.
