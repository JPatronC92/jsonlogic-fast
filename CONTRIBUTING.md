# Contributing to Tempus Engine

First off, thank you for considering contributing to Tempus Engine.

## Code of Conduct

This project and everyone participating in it is governed by our code of conduct: be respectful, be constructive, and help others learn.

## Quick Start for Contributors

### Prerequisites
- Python 3.12+
- Rust 1.75+
- PostgreSQL 16+
- [uv](https://docs.astral.sh/uv/)
- [maturin](https://www.maturin.rs/)
- Docker (for local DB)

### One-time setup

```bash
git clone https://github.com/your-username/tempus-engine.git
cd tempus-engine
make setup
make db-up
cp .env.example .env
make migrate
```

### Daily development loop

```bash
# 1) Create a branch
#    feat/<short-name> | fix/<short-name> | docs/<short-name> | chore/<short-name> | test/<short-name>

# 2) Run local quality gate
make ci-local

# 3) Run API locally
uv run uvicorn src.interfaces.api.main:app --reload
```

## Branch Naming Convention

Use one of the following prefixes:
- `feat/` for new functionality
- `fix/` for bug fixes
- `docs/` for documentation changes
- `chore/` for maintenance work
- `refactor/` for internal restructuring
- `test/` for test-only improvements

Examples:
- `fix/security-import-dedup`
- `docs/api-endpoint-alignment`
- `test/auth-precedence-cases`

## Pull Requests

1. Fork the repository
2. Create a branch following the naming convention above
3. Make your changes
4. Run quality checks locally:
   ```bash
   make lint
   make test
   ```
5. If touching Rust core, run:
   ```bash
   make rust-check
   ```
6. Update docs if behavior or API contract changed
7. Commit with clear message in imperative mood
8. Open a Pull Request using the template

## What reviewers will check

- Correctness and readability
- Test coverage for new behavior
- Backward compatibility (or explicit breaking-change note)
- Documentation updates when routes/contracts/config changed
- No secrets or sensitive data introduced

## How Can I Contribute?

### Reporting Bugs

Before creating bug reports, please check existing issues to avoid duplicates. Include:
- Clear title and description
- Steps to reproduce
- Expected vs actual behavior
- Environment details (OS, Python, Rust, PostgreSQL)
- Relevant logs (redacted)

### Suggesting Enhancements

Enhancement suggestions are tracked as GitHub issues. Include:
- Use case and problem statement
- Proposed solution
- Alternatives considered

## Project Structure

```
tempus-engine/
├── tempus_core/          # Rust core (JSON-Logic engine)
├── src/                  # Python FastAPI application
│   ├── core/             # Config, security, utilities
│   ├── domain/           # Business logic and models
│   ├── infrastructure/   # Database and repositories
│   └── interfaces/       # API routers and dependencies
├── tempus-python/        # Python SDK
├── tempus-node/          # Node.js SDK
├── tempus_wasm/          # WebAssembly module
├── tests/                # Test suite
└── docs/                 # Documentation
```

## Coding Standards

### Python
- Follow PEP 8
- Use type hints
- Max line length: 88
- Required checks:
  ```bash
  make lint
  ```

### Rust
- Use `cargo fmt`, `cargo clippy`, and `cargo test`
- Document public APIs with rustdoc
- Required checks:
  ```bash
  make rust-check
  ```

## Testing

```bash
# Full suite
make test

# Focused checks
make test-fast
```

When adding a change:
- Add or update tests for the modified behavior
- Cover happy path + failure path
- Prefer deterministic fixtures

## Documentation

Update docs whenever you change:
- API routes or payloads
- Security/auth behavior
- Environment variables
- CLI/setup commands

## Security

- Never commit secrets or credentials
- Report vulnerabilities privately (see `SECURITY.md`)
- Prefer least-privilege defaults

## Questions?

Open an issue with the `question` label and enough context.

Thanks again for contributing! 🎉
