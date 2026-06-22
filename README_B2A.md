# jsonlogic-fast B2A Serverless API

This directory contains the B2A (Business-to-Agent) implementation of `jsonlogic-fast`.

## Features
- **Serverless**: Deploys as an AWS Lambda function via `cargo lambda`.
- **SIWE Authentication**: Agents authenticate by signing requests with their Ethereum wallet via SIWE.
- **Cost Estimation**: Endpoints to estimate the cost of an evaluation before running it.
- **Mock Staking/Payment**: Integrates with a mock staking contract for B2A billing per evaluation.

## Smart Contract

There is a simple mock staking contract in `b2a_smart_contract.sol`.

## API Endpoints

### `POST /v1/estimate`
Returns the estimated cost based on rule depth and batch size.

### `POST /v1/evaluate`
Performs the JSONLogic evaluation and subtracts the cost from the sender's balance. Requires a valid SIWE signature.

## Running Locally

Run the local Axum server:
\`\`\`bash
cargo run -p api --bin api_server
\`\`\`

## Deployment to AWS Lambda

\`\`\`bash
cargo lambda build --release --arm64
cargo lambda deploy
\`\`\`
