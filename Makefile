.PHONY: help build test clean install run format lint security doc examples

help:
	@echo "AI Model Vault - Makefile commands"
	@echo ""
	@echo "Development:"
	@echo "  make build      - Build the project"
	@echo "  make test       - Run tests"
	@echo "  make run        - Run the CLI"
	@echo "  make format     - Format code"
	@echo "  make lint       - Run linters"
	@echo ""
	@echo "Security:"
	@echo "  make security   - Run security checks"
	@echo "  make audit      - Run cargo audit"
	@echo ""
	@echo "Documentation:"
	@echo "  make doc        - Generate documentation"
	@echo "  make examples   - Run examples"
	@echo ""
	@echo "Installation:"
	@echo "  make install    - Install aim (AI Model Vault)"
	@echo "  make clean      - Clean build artifacts"

build:
	cargo build --release

build-dev:
	cargo build

test:
	cargo test --all-features

test-verbose:
	cargo test --all-features -- --nocapture

test-integration:
	cargo test --test integration_tests

clean:
	cargo clean
	rm -rf target/

install:
	cargo install --path .

run:
	cargo run --

format:
	cargo fmt

format-check:
	cargo fmt -- --check

lint:
	cargo clippy -- -D warnings

security: audit deny

audit:
	cargo audit

deny:
	cargo deny check

doc:
	cargo doc --no-deps --open

doc-all:
	cargo doc --all-features --open

examples:
	cargo run --example basic_usage
	cargo run --example security_demo

benchmark:
	cargo bench

coverage:
	cargo tarpaulin --out Html --output-dir coverage

watch:
	cargo watch -x build

watch-test:
	cargo watch -x test

all: format lint test build

release: format lint security test
	cargo build --release --all-features

check-all: format-check lint security test
	@echo "All checks passed!"
