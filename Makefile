.PHONY: help setup test clippy lint bench bench-quick bench-python build-py build-wasm test-python test-wasm ci-local
UV_CACHE_DIR ?= .uv-cache

help:
	@echo "Available targets:"
	@echo "  setup       Build local Python bindings"
	@echo "  test        Run core Rust tests"
	@echo "  clippy      Run Rust linter with strict warnings"
	@echo "  bench       Run Criterion benchmarks"
	@echo "  bench-quick Run reduced benchmarks"
	@echo "  bench-python Compare jsonlogic-fast vs json-logic-py"
	@echo "  build-py    Build Python wheel"
	@echo "  build-wasm  Check WASM compilation"
	@echo "  test-python Build + run Python e2e tests"
	@echo "  test-wasm   Run WASM runtime tests in Node.js"
	@echo "  ci-local    Full local quality gate"

setup:
	cd python && UV_CACHE_DIR=$(abspath $(UV_CACHE_DIR)) uv run --with maturin maturin develop --release

test:
	cd core && cargo test --verbose

clippy:
	cargo clippy --all-targets --manifest-path Cargo.toml -- -D warnings

bench:
	cd core && cargo bench --bench bench

bench-quick:
	cd core && cargo bench --bench bench -- --sample-size 10

bench-python:
	cd python && UV_CACHE_DIR=$(abspath $(UV_CACHE_DIR)) uv run --with maturin --with json-logic-qubit bash -c "maturin develop --release && python $(CURDIR)/benchmarks/compare.py"

build-py:
	cd python && UV_CACHE_DIR=$(abspath $(UV_CACHE_DIR)) uv run --with maturin maturin build --release

build-wasm:
	cd wasm && cargo check --target wasm32-unknown-unknown

test-python:
	cd python && UV_CACHE_DIR=$(abspath $(UV_CACHE_DIR)) uv run --with maturin --with pytest bash -c "maturin develop --release && pytest $(CURDIR)/tests/python/ -v"

test-wasm:
	cd wasm && wasm-pack test --node

ci-local: test clippy test-python test-wasm bench-quick
