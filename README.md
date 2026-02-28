# Tempus Rule Engine ⏳⚛️

**The Universal, Deterministic & Time-Travel Compliance Infrastructure.**

[![CI](https://github.com/JPatronC92/Lex-API-Mx/actions/workflows/main.yml/badge.svg)](https://github.com/JPatronC92/Lex-API-Mx/actions/workflows/main.yml)
[![Python 3.12+](https://img.shields.io/badge/python-3.12+-blue.svg)](https://www.python.org/downloads/)
[![FastAPI](https://img.shields.io/badge/FastAPI-0.115+-009688.svg)](https://fastapi.tiangolo.com/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Tempus is a high-performance, domain-agnostic rule engine designed for mission-critical compliance. It treats business logic, legal requirements, and technical constraints as **versioned source code**, allowing you to evaluate transactions against the exact rules that were active at any specific point in history—or even future rules already approved.

---

## 🚀 Why Tempus?

In complex industries (Fintech, Global Trade, Health), rules change constantly. Most systems only know the "current" state of a rule. **Tempus knows the entire timeline.**

### 🌟 Core Superpowers

*   **🕰️ Absolute Time-Travel:** Leveraging PostgreSQL's `DATERANGE` and `ExcludeConstraint` (GiST), Tempus mathematically guarantees that rule versions never overlap. You can audit a 2022 transaction against 2022 rules with 100% certainty.
*   **🧠 Zero-Hallucination Determinism:** Uses `json-logic` for rule execution. Given the same input and date, the output is identical forever. No probabilistic AI "guesses" for compliance.
*   **🛡️ Built-in Input Guard:** Every domain uses dynamic **JSON Schema Contexts** (`EsquemaContexto`) to validate incoming transaction payloads *before* they touch the rule logic, guaranteeing strict structural integrity across any industry.
*   **🌎 Universal Planetary Architecture:** Rules are identified by strict standard **URNs** (`urn:lex:iso:mx:...`) and filtered contextually by *Jurisdictions* and *Authorities* (`AmbitoAplicacion`).
*   **🔒 Cryptographic Event Sourcing:** Every rule version published is hashed with SHA-256 (`hash_firma`). You can irrefutably demonstrate via the `/verify-rule` API that a decision was made using a mathematically unaltered rule.
*   **⚡ Sub-millisecond Memory Cache:** An asynchronous TTL-based memory cache mitigates database round-trips for repetitive context scoping during high-throughput ERP rule evaluations.
*   **🔍 Strict-Time Semantic Search:** Integrated with **Qdrant Vector DB**. Search rules conceptually (e.g., *"Show me import restrictions for microchips"*) while automatically filtering out any rule that wasn't legally active on the target date.

---

## 🏗️ Architecture

Tempus follows a **Clean Architecture** pattern, splitting concerns between two massive operational wings:

```mermaid
graph TD
    %% INGESTION
    A[Rule Definitions / API] -->|LLM/Manual| C{Rule JSON + Metadata}
    C -->|Patcher| D[Temporal Engine]
    D -->|Atomic Update| E[(PostgreSQL Temporal)]
    
    %% VECTORIZATION
    E -->|Background Sync| Q[(Qdrant Vector DB)]
    
    %% CONSUMPTION
    E -->|Deterministic Rules| I[API REST: /evaluate]
    E -->|Audit Trail| K[API REST: /history]
    Q -->|Strict-Time Semantic Search| R[AI Agents / RAG]
    
    %% CLIENTS
    I -->|Yes/No + Details| J[ERP / Banking / Aduanas]
    R -->|Contextual Reasoning| S[AI Compliance Assistants]
```

---

## 🛠️ Tech Stack

*   **Runtime:** Python 3.12+ with `uv` for lightning-fast dependency management.
*   **Web Framework:** FastAPI (Asynchronous).
*   **Primary DB:** PostgreSQL 16 (Temporal ranges & GiST indexes).
*   **Vector DB:** Qdrant (Semantic indexing).
*   **Rule Logic:** `json-logic-qubit`.
*   **LLM Integration:** LiteLLM (Agnostic: OpenAI, Anthropic, DeepSeek).

---

## ⚡ Quick Start

### 1. Prerequisites
* [Docker & Docker Compose](https://docs.docker.com/compose/)
* [uv](https://github.com/astral-sh/uv) (Recommended)

### 2. Infrastructure Setup
```bash
# Clone and enter
git clone https://github.com/JPatronC92/Lex-API-Mx.git && cd Lex-API-Mx

# Start DB and Qdrant
docker-compose up -d

# Install dependencies
uv sync
```

### 3. Initialize & Seed
```bash
# Run migrations to setup temporal constraints
uv run alembic upgrade head

# Seed a sample Universal Rule (e.g., Aduana MEX Microchips)
uv run python scripts/seed_universal_rules.py
```

### 4. Run the API
```bash
uv run uvicorn src.interfaces.api.main:app --reload
```

---

## 🔌 API Showcase

### `/evaluate` (Universal Engine)
Evaluate a transaction against the matrix of rules active at a specific point in time, targeting a specific geopolitical and industrial context.

**POST** `/api/v1/compliance/evaluate`

```json
{
  "selector": {
    "jurisdiccion": "iso:mx",
    "autoridad": "SAT",
    "industria": "FINANZAS",
    "dominio": "FISCAL_RENTA"
  },
  "fecha_operacion": "2026-03-15",
  "contexto": {
    "tipo": "combustible",
    "monto": 3000,
    "metodo_pago": "EFECTIVO"
  }
}
```

**Response (BLOCKER):**
```json
{
  "es_valido": false,
  "reglas_ejecutadas": 1,
  "detalles_fallos": [
    {
      "urn_global": "urn:lex:mx:fiscal:isr:deduccion_combustible",
      "mensaje": "Gasto de combustible por 3000 requiere tarjeta, no EFECTIVO.",
      "criticidad": "BLOCKER"
    }
  ]
}
```

### `/verify-rule/{id}` (Cryptographic Audit)
Verify that a rule's logic hasn't been tampered with since its publication.

**POST** `/api/v1/management/verify-rule/123e4567-e89b-12d3-a456-426614174000`

```json
{
  "valid": true,
  "hash_match": true,
  "stored_hash": "sha256:abcd1234...",
  "recalculated_hash": "sha256:abcd1234...",
  "hash_algoritmo": "SHA-256"
}
```

---

## 🗺️ Roadmap

*   [ ] **Rust Core:** Rewrite the evaluation loop in Rust for sub-millisecond latency at scale.
*   [ ] **Backoffice UI:** Visual timeline for rule management and approval workflows.
*   [ ] **Multi-Tenancy:** Isolated rule-spaces for different organizational departments.

---

## 📄 License
MIT License. Created by [JPatronC92](https://github.com/JPatronC92).
