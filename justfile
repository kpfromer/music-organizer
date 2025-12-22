set dotenv-load := true

import 'backend.just'
import 'frontend.just'

# Watch the backend server, frontend dev, and codegen concurrently
[parallel]
watch: backend-watch frontend-dev

# Build both frontend and backend
[parallel]
build: frontend-build backend-build-release

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

