set dotenv-load := true

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

# Lint Rust code with clippy
lint:
    cargo clippy --all-targets --all-features -- -D warnings

build: build-frontend build-backend

build-backend:
    cargo build --release --locked

[working-directory: 'frontend']
build-frontend:
    bun run build

docker-build:
    docker build -t music-manager:latest .

docker-run:
    docker run -d \
        --name music-manager \
        -p 3000:3000 \
        -e ACOUSTID_API_KEY=${ACOUSTID_API_KEY} \
        -e MUSIC_MANAGER_WATCH_DIRECTORY=/app/music \
        -e MUSIC_MANAGER_CONFIG=/app/config.toml \
        -v ${MUSIC_MANAGER_WATCH_DIRECTORY}:/app/music:ro \
        -v ${MUSIC_MANAGER_CONFIG}:/app/config.toml:ro \
        music-manager:latest \
        serve \
        --port 3000 \
        --directory /app/music

alias c := check

[parallel]
check: check-frontend check-backend

check-backend:
    cargo clippy

[parallel]
check-frontend: check-frontend-biome check-frontend-typescript

[working-directory: 'frontend']
check-frontend-biome:
    bun run check

[working-directory: 'frontend']
check-frontend-typescript:
    bun run check:typescript


[working-directory: 'frontend']
fix-frontend-biome:
    bun run format
    bun run biome check --write ./


just: watch

