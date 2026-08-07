# Contributing

Thank you for your interest in contributing to `jsonlogic-fast`!

## Workflow
1. Fork the repo.
2. Ensure you have Rust and `uv` installed.
3. Run `make setup`.
4. Make your changes in a new branch.
5. Run `make ci` and ensure all checks, tests, and linters pass.
6. Open a PR.

## Code Standards
- All Rust code must pass `cargo clippy` without warnings.
- Documentation should be updated if new features are added.
- Please add tests for any new behavior.

## Release Process

`jsonlogic-fast` implements an automated, secure pipeline to manage releases to crates.io and PyPI:

1. **Version Alignment & Guardrails:**
   - When making changes in critical paths (`core/`, `python/`, or `wasm/`), you **must** bump the workspace version in `Cargo.toml`.
   - You must also update the dependency versions inside `python/Cargo.toml` and `wasm/Cargo.toml`, as well as `version` in `python/pyproject.toml`.
   - A CI guardrail check (`scripts/check_version.py`) automatically ensures that any PR touching these critical paths is blocked if the local version has not been bumped compared to the latest published version on crates.io.

2. **Triggering a Release:**
   - Once your PR with the bumped version is merged into `main`, create and push a git tag matching the version format `v*.*.*` (e.g., `git tag v0.2.0 && git push origin v0.2.0`).

3. **Automated Workflows:**
   - The tag push triggers the `Publish to crates.io` workflow (`publish-crates.yml`).
   - The workflow verifies that the pushed tag matches the package version in `Cargo.toml`.
   - The workspace tests are run in full (`cargo test --workspace --verbose`).
   - If successful, the crate is published to crates.io.
   - Finally, a GitHub Release is automatically created with auto-generated release notes based on commit history.
   - Creating the GitHub Release automatically triggers the `Publish to PyPI` workflow (`publish-pypi.yml`), which builds source distributions and binary wheels for multiple architectures and publishes them to PyPI.
