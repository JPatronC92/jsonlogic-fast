# Tempus Rule Engine ⏳⚛️

**The Universal, Time-Travel Compliance Infrastructure.**

> Formally known as Lex MX Engine, this project has pivoted from a legal-specific tool to a **domain-agnostic, universal rule engine**. 

Tempus Rule Engine allows you to define complex business, legal, or compliance rules (using `json-logic`), version them precisely in time using PostgreSQL's advanced temporal features, and evaluate transactions deterministically against the exact rules that were active at a specific millisecond in history.

It is designed for high-throughput, mission-critical systems like:
* **Global Customs & Logistics:** Validating tariffs and permits (e.g., HS Codes).
* **Fintech & Banking:** Transaction monitoring and dynamic compliance over time.
* **Healthcare:** Evaluating claims against versioned medical policies.
* **LegalTech:** Auditing historical contracts against the law of the time.

## 🌟 Core Superpowers

1. **🕰️ Time-Travel (Temporal Validity):** Rules don't just exist; they exist *in a specific time range*. We use PostgreSQL's `DATERANGE` and `ExcludeConstraint` (GiST indexes) to mathematically guarantee that rule versions never overlap. 
2. **🧠 Deterministic Evaluation:** The `ComplianceEngine` uses `json-logic` to evaluate inputs. Given the same transaction and the same date, the result is mathematically guaranteed to be identical forever. Zero hallucinations.
3. **🔍 Semantic Wing (Qdrant RAG):** Built-in integration with **Qdrant Vector Database**. Texts or human-readable rule definitions are embedded (OpenAI/LiteLLM) along with their temporal boundaries. This allows AI agents to do "Strict-Time Semantic Search" (e.g., *"Find rules about microchip imports active in March 2026"*).

## 🏗️ Architecture

The system is divided into two massive wings:

* **The Deterministic Wing (PostgreSQL + FastAPI):**
  Handles the exact mathematical evaluation of rules. A transaction hits the `/evaluate` endpoint, the engine travels in time to fetch the exact JSON-Logic active on that date, and returns a strict Pass/Fail with detailed error messages.
* **The Semantic Wing (Qdrant Vector DB):**
  A synchronized shadow of the rules engine. It stores vector embeddings of the rules, heavily indexed by `vigencia_inicio` and `vigencia_fin`, enabling AI to search and reason about the rules without hallucinating outdated policies.

## 🚀 Getting Started

### 1. Prerequisites
* [Docker & Docker Compose](https://docs.docker.com/compose/)
* Python 3.12+ (We recommend using `uv` for dependency management)

### 2. Environment Setup
Copy the example environment file and fill in your API keys (especially `LLM_API_KEY` for vectorization):
```bash
cp .env.example .env
```

### 3. Spin up the Infrastructure
Start PostgreSQL (with `btree_gist` extension) and Qdrant:
```bash
docker-compose up -d
```

### 4. Install Dependencies
```bash
uv sync
# Or using standard pip: pip install -r pyproject.toml
```

### 5. Run Database Migrations
Set up the tables and the temporal exclusion constraints:
```bash
alembic upgrade head
```

### 6. Seed Universal Rules & Start Server
Inject the sample Universal Rule (e.g., Taiwan Microchip Customs Rule):
```bash
python scripts/seed_universal_rules.py
```

Start the FastAPI server:
```bash
uvicorn src.interfaces.api.main:app --reload
```

## 🔌 API Usage Example

**Endpoint:** `POST /api/v1/compliance/evaluate`

**Payload:**
```json
{
  "transaccion": {
    "codigo_hs": "8542.31",
    "origen": "TWN",
    "destino": "MEX",
    "valor_usd": 50000,
    "tiene_certificado_nom": false
  },
  "fecha_operacion": "2026-03-15"
}
```

**Response:**
```json
{
  "es_valido": false,
  "score_cumplimiento": 0.0,
  "errores": [
    "El embarque de TWN requiere Certificado NOM-019 porque su valor ($ 50000) supera los $1000 USD."
  ],
  "warnings": [],
  "reglas_ejecutadas": 1,
  "detalles_fallos": [
    {
      "clave": "ADUANA-MEX-8542-NOM",
      "severidad": "BLOCKER",
      "mensaje": "El embarque de TWN requiere Certificado NOM-019 porque su valor ($ 50000) supera los $1000 USD."
    }
  ]
}
```
