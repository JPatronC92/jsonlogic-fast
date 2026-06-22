import requests
from eth_account import Account
from eth_account.messages import encode_defunct
from datetime import datetime, timezone
from urllib.parse import urlparse
import json

def create_siwe_message(address: str, domain: str, uri: str, chain_id: int) -> str:
    """Create a basic EIP-4361 SIWE message"""
    now = datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")
    return (
        f"{domain} wants you to sign in with your Ethereum account:\n"
        f"{address}\n\n"
        f"Sign in to jsonlogic-fast B2A Serverless API.\n\n"
        f"URI: {uri}\n"
        f"Version: 1\n"
        f"Chain ID: {chain_id}\n"
        f"Nonce: random-nonce-123456\n"
        f"Issued At: {now}"
    )

def main():
    # 1. Setup Ethereum Account (Mock Agent Wallet)
    account = Account.create()
    print(f"Agent Wallet Address: {account.address}")

    domain = "localhost:3000"
    uri = "http://localhost:3000/v1/evaluate"
    chain_id = 1

    # 2. Create and sign SIWE message
    siwe_message = create_siwe_message(account.address, domain, uri, chain_id)
    signable_message = encode_defunct(text=siwe_message)
    signed_message = account.sign_message(signable_message)
    signature_hex = signed_message.signature.hex()

    print(f"\nSigned SIWE Message!")

    # 3. Define JSONLogic rule and data
    rule = {"+": [{"var": "a"}, {"var": "b"}]}
    data = {"a": 100, "b": 250}

    # 4. First, estimate cost
    estimate_url = "http://localhost:3000/v1/estimate"
    est_payload = {"rule_depth": 1, "batch_size": 1}
    try:
        est_res = requests.post(estimate_url, json=est_payload)
        print(f"\nEstimated Cost: ${est_res.json()['estimated_cost']}")
    except Exception as e:
        print(f"Failed to estimate: {e}")

    # 5. Execute Evaluation
    payload = {
        "message": siwe_message,
        "signature": signature_hex,
        "rule": rule,
        "data": data
    }

    try:
        res = requests.post(uri, json=payload)
        print(f"\nEvaluation Result: {json.dumps(res.json(), indent=2)}")
    except Exception as e:
        print(f"Failed to evaluate: {e}")

if __name__ == "__main__":
    main()
