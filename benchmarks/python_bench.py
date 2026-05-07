import json
import timeit
import argparse
from typing import List

try:
    import jsonlogic_fast
except ImportError:
    print("Error: jsonlogic-fast not installed. Run `uv pip install -e ./python`")
    exit(1)

# Generate dummy data
def generate_contexts(n: int) -> List[str]:
    contexts = []
    for i in range(n):
        ctx = {"score": i % 1000, "user_id": f"u_{i}", "active": i % 2 == 0}
        contexts.append(json.dumps(ctx))
    return contexts

def run_benchmarks():
    parser = argparse.ArgumentParser(description="Benchmark jsonlogic-fast against json-logic-qubit")
    parser.add_argument("--sizes", type=int, nargs="+", default=[1000, 10000, 100000], help="Batch sizes to test")
    args = parser.parse_args()

    rule = json.dumps({"if": [{">": [{"var": "score"}, 500]}, True, False]})
    compiled = jsonlogic_fast.CompiledRule(rule)

    print("--- Python Benchmark for jsonlogic-fast ---")

    for size in args.sizes:
        print(f"\nGenerating {size} contexts...")
        contexts = generate_contexts(size)

        # Test sequential baseline (if we evaluate them one by one in Python)
        start = timeit.default_timer()
        for ctx in contexts:
            jsonlogic_fast.evaluate(rule, ctx)
        seq_time = timeit.default_timer() - start

        # Test batch evaluate
        start = timeit.default_timer()
        jsonlogic_fast.evaluate_batch(rule, contexts)
        batch_time = timeit.default_timer() - start

        # Test compiled evaluate loop
        start = timeit.default_timer()
        for ctx in contexts:
            compiled.evaluate(ctx)
        compiled_time = timeit.default_timer() - start

        # Test compiled batch evaluate
        start = timeit.default_timer()
        compiled.evaluate_batch(contexts)
        compiled_batch_time = timeit.default_timer() - start

        print(f"Batch Size: {size}")
        print(f"  Sequential evaluate: {seq_time*1000:.2f} ms")
        print(f"  Compiled sequential: {compiled_time*1000:.2f} ms ({(seq_time/compiled_time):.1f}x vs sequential)")
        print(f"  String batch:        {batch_time*1000:.2f} ms ({(seq_time/batch_time):.1f}x vs sequential)")
        print(f"  Compiled batch:      {compiled_batch_time*1000:.2f} ms ({(seq_time/compiled_batch_time):.1f}x vs sequential)")

if __name__ == "__main__":
    run_benchmarks()
