import json
from jsonlogic_fast import CompiledRule

def main():
    print("--- Credit Eligibility Evaluation (Python) ---")

    # Complex rule for credit card approval
    rule_json = json.dumps({
        "and": [
            {">=": [{"var": "credit_score"}, 700]},
            {"<=": [{"var": "debt_to_income_ratio"}, 0.4]},
            {">=": [{"var": "annual_income"}, 50000]}
        ]
    })

    # Compile rule once
    rule = CompiledRule(rule_json)

    applicants = [
        {"name": "Alice", "credit_score": 750, "debt_to_income_ratio": 0.3, "annual_income": 60000},
        {"name": "Bob", "credit_score": 680, "debt_to_income_ratio": 0.2, "annual_income": 80000},
        {"name": "Charlie", "credit_score": 720, "debt_to_income_ratio": 0.5, "annual_income": 55000},
    ]

    for applicant in applicants:
        ctx_json = json.dumps(applicant)
        is_eligible = rule.evaluate(ctx_json)
        print(f"Applicant {applicant['name']} eligible: {is_eligible}")

    # Batch evaluate
    print("\nBatch evaluation:")
    contexts = [json.dumps(a) for a in applicants]
    results = rule.evaluate_batch(contexts)
    for name, res in zip([a['name'] for a in applicants], results):
        print(f"{name}: {res}")

if __name__ == "__main__":
    main()
