# Watch the backend server, frontend dev, and codegen concurrently
[parallel]
watch: watch-backend dev-frontend

# Watch only the backend server
watch-backend:
    cargo watch -x 'run -- serve --log-level info --directory downloads'

# Watch/develop the frontend
[working-directory: 'frontend']
dev-frontend: 
    bun run dev

# Run GraphQL codegen for the frontend
[working-directory: 'frontend']
codegen-frontend: 
    bun run codegen

# Check frontend code with Biome
[working-directory: 'frontend']
check-frontend: 
    bun run check

# Lint Rust code with clippy
lint:
    cargo clippy --all-targets --all-features -- -D warnings

build: build-frontend build-backend

build-backend:
    cargo build --release --locked

[working-directory: 'frontend']
build-frontend:
    bun run build

just: watch