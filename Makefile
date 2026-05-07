.PHONY: setup test test-python test-wasm bench bench-quick bench-python ci fmt clippy build-py

setup:
	cargo check
	python3 -m pip install uv
	uv pip install -e ./python

test:
	cargo test --workspace

test-python:
	uv run --with maturin --with pytest bash -c "cd python && maturin develop --release && pytest ../tests/python/ -v"

test-wasm:
	cd wasm && wasm-pack test --node

bench:
	cargo bench --manifest-path core/Cargo.toml

bench-quick:
	cargo bench --manifest-path core/Cargo.toml --bench bench -- --sample-size 10

bench-python:
	uv run --with maturin bash -c "python3 benchmarks/python_bench.py"

fmt:
	cargo fmt --all

clippy:
	cargo clippy --workspace --all-targets -- -D warnings

build-py:
	uv run --with maturin bash -c "cd python && maturin build --release"

ci: fmt clippy test test-python test-wasm
