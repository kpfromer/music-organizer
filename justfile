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



just: watch

