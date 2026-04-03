#!/usr/bin/env python3
"""
Batch evaluation example.

    pip install jsonlogic-fast
    python examples/python/batch.py
"""

import json
import jsonlogic_fast


def main() -> int:
    rule = {
        "if": [
            {">": [{"var": "amount"}, 1000]},
            {"*": [{"var": "amount"}, 0.025]},
            {"*": [{"var": "amount"}, 0.035]},
        ]
    }

    contexts = [
        {"amount": 250},
        {"amount": 2000},
        {"amount": 110},
        {"amount": 5000},
    ]

    results = jsonlogic_fast.evaluate_batch_json(
        json.dumps(rule),
        [json.dumps(ctx) for ctx in contexts],
    )

    print("Results:", results)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
