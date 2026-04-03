#!/usr/bin/env python3
"""
Basic evaluation example.

    pip install jsonlogic-fast
    python examples/python/evaluate.py
"""

import json
import jsonlogic_fast


def main() -> int:
    rule = {
        "if": [
            {">": [{"var": "score"}, 700]},
            "approve",
            "review",
        ]
    }
    context = {"score": 742, "country": "MX"}

    result = jsonlogic_fast.evaluate_json(json.dumps(rule), json.dumps(context))

    print("Rule:", json.dumps(rule))
    print("Context:", json.dumps(context))
    print("Result:", result)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
