# Builder stage - use Alpine to avoid OpenSSL compatibility issues
FROM alpine:latest AS builder

# Install build dependencies
RUN apk add --no-cache \
    musl-dev \
    openssl-dev \
    openssl-libs-static \
    pkgconfig \
    build-base \
    curl

# Install Rust via rustup
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable
ENV PATH="/root/.cargo/bin:${PATH}"

WORKDIR /app

# Copy Cargo files for dependency caching
COPY Cargo.toml Cargo.lock ./
COPY migration/Cargo.toml ./migration/

# Create dummy source files for dependency caching
RUN mkdir -p src migration/src && \
    echo "fn main() {}" > src/main.rs && \
    echo "fn main() {}" > migration/src/main.rs && \
    echo "" > migration/src/lib.rs

# Build dependencies (this layer will be cached if Cargo files don't change)
RUN cargo build --release --locked || true

# Copy actual source code
COPY src ./src
COPY migration/src ./migration/src

# Build the application (Alpine uses musl by default)
RUN cargo build --release --locked

# Runtime stage
FROM alpine:latest

# Install runtime dependencies (chromaprint for fpcalc)
RUN apk add --no-cache \
    chromaprint \
    ca-certificates

WORKDIR /app

COPY --from=builder /app/target/release/music-manager /usr/local/bin/music-manager

ENTRYPOINT ["music-manager"]
CMD ["watch"]