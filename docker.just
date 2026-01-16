docker-build:
    docker build -t music-manager:latest .
    @echo "Docker image built successfully"

docker-remove:
    docker rm -f music-manager

docker-test:
    #!/usr/bin/env bash
    set -euo pipefail
    
    TEST_DIR=".docker-test"
    
    # Create tmp directory structure
    mkdir -p "${TEST_DIR}/music" "${TEST_DIR}/downloads" "${TEST_DIR}/config"
    
    # Create config file in tmp folder
    cat > "${TEST_DIR}/config/config.toml" << 'EOF'
    directory = '/app/music'
    database_path = '/app/library.db'
    EOF
    
    # Run docker container with the setup ( fresh DB on each boot)
    docker run --rm -it \
        -p 3005:3000 \
        -e MUSIC_MANAGER_HTTP_PORT=3000 \
        -e BASE_URL=http://localhost:3000 \
        -e MUSIC_MANAGER_WATCH_DIRECTORY=/app/music \
        -e SOULSEEK_DOWNLOAD_DIRECTORY=/app/downloads \
        -e ACOUSTID_API_KEY="${ACOUSTID_API_KEY:-test-key}" \
        -e SOULSEEK_USERNAME="${SOULSEEK_USERNAME:-test-user}" \
        -e SOULSEEK_PASSWORD="${SOULSEEK_PASSWORD:-test-password}" \
        -e MUSIC_MANAGER_CONFIG=/app/config/config.toml \
        -e LOG_LEVEL=debug \
        -v "${TEST_DIR}/music:/app/music" \
        -v "${TEST_DIR}/downloads:/app/downloads" \
        -v "${TEST_DIR}/config:/app/config" \
        music-manager:latest serve