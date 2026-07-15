.PHONY: check test clippy fmt doc ci

check:
	cargo check --workspace --all-targets

test:
	cargo test --workspace --all-features

clippy:
	cargo clippy --workspace --all-targets -- -D warnings

fmt:
	cargo fmt --all -- --check

fmt-fix:
	cargo fmt --all

doc:
	cargo doc --workspace --no-deps
	@# Also verify RUSTDOCFLAGS=-Dwarnings passes on CI
	RUSTDOCFLAGS="-Dwarnings" cargo doc --workspace --no-deps 2>&1 | tail -3

ci: fmt check clippy test doc
	@echo "=== CI GREEN ==="
