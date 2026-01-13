# Format Rust code
backend-format:
    cargo fmt

# Check Rust formatting
backend-format-check:
    cargo fmt --check

# Run clippy linter with strict warnings
backend-lint:
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
    if ! command -v cargo-audit > /dev/null; then cargo install cargo-audit --locked; fi
    cargo audit --ignore RUSTSEC-2023-0071 --ignore RUSTSEC-2026-0001 --ignore RUSTSEC-2026-0002

# Watch backend server
backend-watch:
    cargo watch -w src -x 'run -- serve --log-level info --directory downloads'

# Check (format-check + lint + test)
[parallel]
backend-check: backend-format-check backend-lint backend-test

