# Chef stage - install cargo-chef
FROM alpine:latest AS chef

# Install build dependencies
RUN apk add --no-cache \
    musl-dev \
    openssl-dev \
    openssl-libs-static \
    pkgconfig \
    build-base \
    curl

# Install Rust via rustup
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable --profile minimal
ENV PATH="/root/.cargo/bin:${PATH}"

# Install cargo-chef
RUN cargo install cargo-chef --locked

WORKDIR /app

# Frontend builder stage - use Bun image
FROM oven/bun:latest AS frontend-builder
WORKDIR /app
COPY frontend/package.json frontend/bun.lock* ./
RUN bun install --frozen-lockfile
COPY frontend/ ./
RUN bun run build

# Planner stage - generate recipe.json
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# Builder stage - build dependencies then application
FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json

# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json

# Copy source code
COPY . .

# Copy frontend dist from frontend-builder stage
COPY --from=frontend-builder /app/dist /app/frontend/dist

# Build Rust application (release mode requires frontend/dist to exist)
RUN cargo build --release --locked

# Runtime stage
FROM alpine:latest

# Install runtime dependencies (chromaprint for fpcalc)
RUN apk add --no-cache \
    chromaprint \
    ca-certificates

WORKDIR /app

COPY --from=builder /app/target/release/music-manager /usr/local/bin/music-manager
COPY --from=builder /app/frontend/dist /app/frontend/dist

ENTRYPOINT ["music-manager"]
CMD ["watch"]