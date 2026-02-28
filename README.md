# Tempus Billing & Commission Engine ⏳💰

**The Deterministic & Time-Travel Pricing Infrastructure for Fintechs and Marketplaces.**

[![CI](https://github.com/JPatronC92/Lex-API-Mx/actions/workflows/main.yml/badge.svg)](https://github.com/JPatronC92/Lex-API-Mx/actions/workflows/main.yml)
[![Python 3.12+](https://img.shields.io/badge/python-3.12+-blue.svg)](https://www.python.org/downloads/)
[![FastAPI](https://img.shields.io/badge/FastAPI-0.115+-009688.svg)](https://fastapi.tiangolo.com/)

Tempus is a high-performance, domain-agnostic pricing engine designed for mission-critical financial systems. It treats fees, commission splits, and pricing tiers as **versioned mathematical rules**, allowing you to calculate or audit a transaction against the exact pricing scheme that was active at any specific point in history.

Stop burying your pricing logic in spaghetti `if/else` backend code. 

---

## 🚀 Why Tempus?

In Fintech, SaaS, and Marketplaces, pricing rules change constantly. You have multiple tiers, volume-based fees, contractual overrides, and temporary promotions. Most billing systems fail when asked: *"How was this commission calculated 2 years ago?"* 

**Tempus guarantees deterministic, auditable, and time-travel-ready financial operations.**

### 🌟 Core Superpowers

*   **🕰️ Absolute Time-Travel Audit:** Leveraging PostgreSQL's `DATERANGE` and `ExcludeConstraint` (GiST), Tempus mathematically guarantees that pricing rule versions never overlap. 
*   **🧠 Zero-Float Determinism:** Uses `json-logic` for rule execution. Given the same input payload and historical date, the fee output is identical forever.
*   **🛡️ Built-in Payload Guard:** Every pricing scheme uses dynamic **JSON Schema Contexts** to validate incoming transaction payloads *before* they touch the math logic, preventing costly data errors.
*   **🔒 Cryptographic Receipt:** Every calculation returns a SHA-256 hash of the specific rule versions used. You can irrefutably demonstrate to auditors or merchants exactly how a fee was generated.
*   **📊 Mass Simulation (Batch-Audit):** Test new pricing tiers against millions of historical transactions *before* deploying them to production to forecast revenue impact.

### 🦀 Performance (Rust Native Core)
Tempus is written in Python for developer speed, but its mathematical heart beats in **Rust**. By using `PyO3` and `maturin`, the engine delegates all determinist math to a pre-compiled Rust extension (`tempus_core`), capable of pushing **+330,000 evaluations per second** on a single thread.

---

## 🏗️ The Pricing Model

Tempus abstract billing into 4 core concepts:

1. **`PricingScheme`**: A collection of rules (e.g., "Marketplace Standard MX").
2. **`PricingRuleIdentity`**: The logical concept of a fee (e.g., "Credit Card Processing Fee").
3. **`PricingRuleVersion`**: The actual mathematical formula (json-logic) and the exact `DATERANGE` it is legally active.
4. **`PricingContextSchema`**: The JSON Schema defining the required payload (amount, currency, tier, country).

---

## ⚡ Quick Start

### 1. Evaluate a Transaction (`/calculate-fee`)

Send a transaction payload and an execution date. The engine will automatically find the correct mathematical rules active on that date and execute them.

**POST** `/api/v1/billing/calculate`

```json
{
  "scheme_urn": "urn:pricing:marketplace:mx",
  "execution_date": "2024-06-15T14:30:00Z",
  "transaction": {
    "amount": 15000.00,
    "currency": "MXN",
    "payment_method": "CREDIT_CARD",
    "merchant_tier": "ENTERPRISE"
  }
}
```

**Deterministic Response:**
```json
{
  "base_amount": 15000.00,
  "calculated_fees": [
    {
      "rule_id": "8f43b2...",
      "name": "Comisión Base 1.5%",
      "amount": 225.00
    },
    {
      "rule_id": "9a12c4...",
      "name": "Fijo por Transacción (MXN)",
      "amount": 3.00
    }
  ],
  "total_fees": 228.00,
  "net_settlement": 14772.00,
  "currency": "MXN",
  "cryptographic_hash": "sha256:abcd1234efgh5678..."
}
```

---

## 🗺️ Strategic Roadmap

*   [x] **API-First Refactor:** Upgrade all controllers and repositories to support the new financial domain models.
*   [x] **Batch Simulation Endpoint:** API for CFOs to send 1M transactions against a draft `PricingScheme` to forecast revenue.
*   [x] **Rust Core Migration:** Rewrite the evaluation engine in Rust for sub-millisecond latency to handle 100k+ TPS (High-Frequency Trading / Stripe-scale).
*   [ ] **Embedded SDKs:** Python and Node.js clients for instant integration.
*   [ ] **Self-Hosted Financial UI:** A React dashboard for finance teams to manage tiers and visualize the time-travel timeline.

---

## 📄 License
MIT License. Created by [JPatronC92](https://github.com/JPatronC92).
