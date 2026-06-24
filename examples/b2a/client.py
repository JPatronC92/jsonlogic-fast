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

def run_stress(base_url: str, num_requests: int = 20, batch_size: int = 5):
    """Fase 4 stress simulation: batch of evaluations with timing (for pre-prod load hints)."""
    import time
    account = Account.create()
    domain = "localhost:3000"
    uri = f"{base_url}/v1/evaluate"
    chain_id = 1

    siwe_message = create_siwe_message(account.address, domain, uri, chain_id)
    signable_message = encode_defunct(text=siwe_message)
    signed_message = account.sign_message(signable_message)
    signature_hex = signed_message.signature.hex()

    rule = {"+": [{"var": "score"}, {"var": "volume"}]}
    data = {"score": 42, "volume": 1000}

    payload = {
        "message": siwe_message,
        "signature": signature_hex,
        "rule": rule,
        "data": data
    }

    latencies = []
    successes = 0
    for i in range(num_requests):
        start = time.time()
        try:
            r = requests.post(uri, json=payload, timeout=10)
            lat = (time.time() - start) * 1000
            latencies.append(lat)
            if r.status_code == 200:
                successes += 1
            print(f"[{i+1}/{num_requests}] {lat:.1f}ms status={r.status_code}")
        except Exception as e:
            print(f"[{i+1}] error: {e}")
        time.sleep(0.05)  # tiny pacing

    if latencies:
        avg = sum(latencies) / len(latencies)
        p95 = sorted(latencies)[int(len(latencies)*0.95)]
        print(f"\nStress summary: {successes}/{num_requests} ok, avg={avg:.1f}ms p95={p95:.1f}ms")


if __name__ == "__main__":
    import sys
    if len(sys.argv) > 1 and sys.argv[1] == "--stress":
        base = sys.argv[2] if len(sys.argv) > 2 else "http://localhost:3000"
        n = int(sys.argv[3]) if len(sys.argv) > 3 else 20
        run_stress(base, n)
    else:
        main()
