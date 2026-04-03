# jsonlogic-fast — Python Bindings

Native Python bindings for the `jsonlogic-fast` JSON-Logic evaluator, built with PyO3.

## Install

```bash
pip install jsonlogic-fast
```

## Usage

```python
import json
import jsonlogic_fast

rule = json.dumps({">": [{"var": "score"}, 700]})
ctx = json.dumps({"score": 742})

result = jsonlogic_fast.evaluate(rule, ctx)  # True
```
