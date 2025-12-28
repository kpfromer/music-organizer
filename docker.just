docker-build:
    docker build -t music-manager:latest .
    @echo "Docker image built successfully"

docker-remove:
    docker rm -f music-manager