.PHONY: help setup db-up db-down migrate test test-fast lint format rust-check ci-local

help:
	@echo "Available targets:"
	@echo "  setup      Install dependencies and build Rust extension"
	@echo "  db-up      Start local PostgreSQL via docker compose"
	@echo "  db-down    Stop local PostgreSQL containers"
	@echo "  migrate    Run database migrations"
	@echo "  test       Run full Python test suite"
	@echo "  test-fast  Run focused auth/pricing tests"
	@echo "  lint       Run black, isort, and ruff checks"
	@echo "  format     Apply black and isort formatting"
	@echo "  rust-check Run fmt, clippy and tests for Rust core"
	@echo "  ci-local   Run local pre-PR quality gate"

setup:
	uv sync
	cd tempus_core && maturin develop --release

db-up:
	docker compose up db -d

db-down:
	docker compose down

migrate:
	uv run alembic upgrade head

test:
	PYTHONPATH=. uv run pytest -v

test-fast:
	PYTHONPATH=. uv run pytest -v tests/test_security.py tests/test_pricing_engine.py

lint:
	uv run black --check src tests
	uv run isort --check-only src tests
	uv run ruff check src tests

format:
	uv run black src tests
	uv run isort src tests

rust-check:
	cd tempus_core && cargo fmt -- --check && cargo clippy -- -D warnings && cargo test --verbose

ci-local: lint test rust-check
