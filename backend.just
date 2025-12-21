# Format Rust code
backend-fmt:
    cargo fmt

# Check Rust formatting
backend-fmt-check:
    cargo fmt --check

# Run clippy linter
backend-clippy:
    cargo clippy

# Run clippy with strict warnings
backend-clippy-strict:
    cargo clippy --all-targets --all-features -- -D warnings

# Run tests
backend-test:
    cargo test

# Build in debug mode
backend-build:
    cargo build

# Build in release mode
backend-build-release:
    cargo build --release --locked

# Run cargo audit
backend-audit:
    cargo install cargo-audit --locked
    cargo audit --ignore RUSTSEC-2023-0071

# Watch backend server
backend-watch:
    cargo watch -x 'run -- serve --log-level info --directory downloads'

# Lint (alias for clippy-strict)
backend-lint: backend-clippy-strict

# Check (alias for fmt-check + clippy-strict + test)
[parallel]
backend-check: backend-fmt-check backend-clippy-strict backend-test

