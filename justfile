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

just: watch

