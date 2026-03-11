import json
import time

from json_logic import jsonLogic

import tempus_core
from json_logic import jsonLogic

# 1. Preparar Datos de Prueba (Simulando 10,000 Transacciones)
NUM_TRANSACTIONS = 10000

# Regla de Stripe: if country == MX then amount * 0.036 else 0
logica_json = {
    "if": [{"==": [{"var": "country"}, "MX"]}, {"*": [{"var": "amount"}, 0.036]}, 0]
}

logica_str = json.dumps(logica_json)

# Creamos 10k transacciones (mitad MX, mitad US)
transacciones = [
    {"amount": 1500.00, "country": "MX" if i % 2 == 0 else "US"}
    for i in range(NUM_TRANSACTIONS)
]

print(f"🚀 Iniciando Benchmark de Tempus Core: {NUM_TRANSACTIONS} transacciones\n")

# --- BENCHMARK 1: PYTHON PURO (json-logic original) ---
start_time = perf_counter()
python_total = 0.0

for tx in transacciones:
    fee = float(jsonLogic(logica_json, tx))
    python_total += fee

python_duration = perf_counter() - start_time
print("🐍 Vía Lenta (Python json-logic):")
print(f"   -> Tiempo total: {python_duration:.4f} segundos")
print(
    f"   -> TPS (Transacciones por segundo): {NUM_TRANSACTIONS / python_duration:,.0f} req/s\n"
)


# --- BENCHMARK 2: RUST NATIVO (tempus_core + PyO3) ---
start_time = perf_counter()
rust_total = 0.0

# Pre-parseamos los JSON a strings para el puente FFI (como lo hace el PricingEngine real)
tx_strings = [json.dumps(tx) for tx in transacciones]

for tx_str in tx_strings:
    fee = tempus_core.evaluate_fee(logica_str, tx_str)
    rust_total += fee

rust_duration = perf_counter() - start_time
print("🦀 Vía Rápida Individual (Rust tempus_core):")
print(f"   -> Tiempo total: {rust_duration:.4f} segundos")
print(
    f"   -> TPS (Transacciones por segundo): {NUM_TRANSACTIONS / rust_duration:,.0f} req/s\n"
)

# --- BENCHMARK 3: RUST BATCH (Array-based FFI) ---
start_time = perf_counter()

# Pasamos toda la lista de strings de una sola vez
batch_results = tempus_core.evaluate_batch(logica_str, tx_strings)
rust_batch_total = sum(batch_results)

rust_batch_duration = perf_counter() - start_time
print("🚀🚀 Vía Ultra Rápida Batch (Rust tempus_core Arrays):")
print(f"   -> Tiempo total: {rust_batch_duration:.4f} segundos")
print(
    f"   -> TPS (Transacciones por segundo): {NUM_TRANSACTIONS / rust_batch_duration:,.0f} req/s\n"
)

# --- RESULTADOS ---
assert (
    python_total == rust_total == rust_batch_total
), "Error: ¡Los motores no dieron el mismo resultado determinista!"

if rust_batch_duration < python_duration:
    speedup = python_duration / rust_batch_duration
    print(f"✅ ¡RUST BATCH ES {speedup:.1f}x MÁS RÁPIDO QUE PYTHON!")
else:
    print("⚠️ Resultado inesperado. Python fue más rápido (revisar overhead de FFI).")
