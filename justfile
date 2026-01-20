set dotenv-load := true

import 'backend.just'
import 'frontend.just'
import 'migrations.just'
import 'docker.just'

# Watch the backend server, frontend dev, and codegen concurrently
[parallel]
watch: backend-watch frontend-dev

# Build both frontend and backend
[parallel]
build: frontend-build backend-build-release

alias c := check

# Run all checks (frontend and backend)
[parallel]
check: frontend-check backend-check

# Lint both frontend and backend
[parallel]
lint: frontend-lint backend-lint

# Format both frontend and backend
[parallel]
format: frontend-format backend-format

# Run tests for backend
test: backend-test

# Fix frontend biome issues
fix-frontend: frontend-fix

# Run codemod to replace log:: with tracing::
codemod-log-to-tracing:
    #!/usr/bin/env bash
    if ! command -v ast-grep > /dev/null; then
        echo "ast-grep is not installed. Install it with:"
        echo "  cargo install ast-grep --locked"
        echo "  or: brew install ast-grep"
        echo "  or: npm install --global @ast-grep/cli"
        exit 1
    fi
    ast-grep scan --rule log-to-tracing.yml --rewrite src/

just: watch

