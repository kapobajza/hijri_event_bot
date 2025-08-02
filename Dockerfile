FROM rust:1.87-slim AS builder

WORKDIR /app

RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libpq-dev \
    protobuf-compiler \
    && rm -rf /var/lib/apt/lists/*

# Copy manifest files for dependency caching
COPY Cargo.toml Cargo.lock .sqlx ./

RUN echo "Datbase URL: ${DATABASE_URL} $DATABASE_URL"

# Create a dummy main.rs to build dependencies
RUN mkdir -p src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -rf src

# Copy actual source code
COPY . .

# Build the application
RUN cargo build --release

# Runtime stage
FROM rust:1.87-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    libssl-dev \
    libpq-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the binary from the builder stage
COPY --from=builder /app/target/release/hijri_event_bot /app/hijri_event_bot

# Copy migrations folder for sqlx
COPY --from=builder /app/migrations /app/migrations

# Copy locale files
COPY --from=builder /app/locales /app/locales

CMD ["./hijri_event_bot"]