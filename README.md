# Tempus Engine ⏳

> **A deterministic decision engine for complex rule evaluation — compiled to WebAssembly, powered by Rust.**

Tempus evaluates any business rule — financial, operational, or regulatory — with **zero floating-point ambiguity**, **sub-millisecond latency**, and **immutable audit trails**. Same input → same output. Always.

---

## 🚀 Live Demo — Public Simulator

Try the interactive decision engine in your browser, no account required:

**[→ tempus-simulator.vercel.app](https://tempus-simulator.vercel.app)**

Select a use-case template and see the engine in action:

| Template | Domain |
|---|---|
| 🏪 Marketplace 3-Tier | Dynamic commission engine |
| 🛡️ AI Agent Guardrails | LLM safety & prompt filtering |
| ⚡ Dynamic API Rate Limiting | CPU-adaptive infrastructure control |

---

## 🧠 What is the Tempus Engine?

Tempus is not a pricing tool — it's a **universal rule compiler** that transforms decision logic into deterministic WebAssembly operations.

```
Your Rule (JSON-Logic) + Your Data → Deterministic Output (every time)
```

Use cases:
- **Fintech:** Marketplace commissions, dynamic pricing, revenue forecasting
- **AI Safety:** Prompt guardrails, token limits, risk thresholds
- **DevOps:** Adaptive rate limiting, SLA-based traffic routing
- **Compliance:** Regulatory rule enforcement, audit trails

---

## 🏗️ Architecture

The engine is built with a clean 3-layer template model:

```
Template
├── dataset          # Input data type (transaction, prompt, api_request)
├── rule_schema      # JSON-Logic rule with editable parameters
└── presentation_profile  # Per-language UI context (en / es)
```

**Engine stack:**
- 🦀 **Rust** — Zero-cost abstractions, memory-safe computation
- ⚙️ **WebAssembly** — Runs in-browser and on edge nodes
- 📐 **JSON-Logic** — Declarative, portable, auditable rules
- ⚛️ **Next.js 16** — Performant React UI (App Router)

---

## ✨ Key Properties

| Property | What it means |
|---|---|
| **Deterministic** | Same input → same output. Always. Mathematical guarantee. |
| **Versioned** | Rules are immutable snapshots. Roll back any decision to any point in history. |
| **Explainable** | Every decision includes a full audit trail — which rule hit which event and why. |
| **Portable** | Compile once to WASM. Run in a browser, a Lambda, or an edge node. |
| **Multilingual** | Full `EN` / `ES` UI support. Technical terms preserved. |

---

## 📂 Repository Structure

```
Tempus-Engine/
├── tempus-dashboard/     # Next.js public simulator (public-sim branch)
│   └── src/
│       ├── app/          # Main simulator UI (page.tsx, simulator.module.css)
│       ├── data/         # Template definitions (3-layer architecture)
│       └── lib/          # WASM bridge, i18n dictionary
├── tempus_core/          # Rust JSON-Logic evaluation core
├── tempus_wasm/          # WASM bindings (wasm-bindgen)
├── tempus-node/          # Node.js SDK
├── tempus-python/        # Python SDK
├── src/                  # B2B API (FastAPI)
└── docs/                 # Technical documentation
```

---

## 🌐 Internationalization

The simulator ships with full `EN` / `ES` support via an internal `i18n.ts` dictionary.
Technical terms (`WASM`, `Throughput`, `Tokens`, `Marketplaces`) are preserved in both languages.

---

## 💳 Commercial Licensing

Tempus is proprietary software. Contact for licensing, custom deployments, or integrations:

**[→ Open an inquiry](https://github.com/JPatronC92/Tempus-Engine/issues)**

| Tier | Best for |
|---|---|
| **Tempus Cloud (SaaS)** | Startups & scale-ups — pay-per-calculation API |
| **Tempus Enterprise** | Banks, Marketplaces — on-premise annual license |
| **Visual Rule Suite** | No-code rule builder + CFO dashboard |

---

## 🔒 Security

- `.env` files are never committed (see `.env.example` for required vars)
- Secrets are managed via environment variables only
- WASM binary is sandboxed and has no file system access

---

> © 2026 JPatronC92. All rights reserved. Proprietary Commercial License.
