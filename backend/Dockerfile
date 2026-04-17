# ── Build stage ──────────────────────────────────────
FROM rust:1.78-bookworm AS builder

WORKDIR /app
COPY Cargo.toml Cargo.lock* ./
# Create dummy main for dependency caching
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release 2>/dev/null || true
RUN rm -rf src

COPY . .
RUN cargo build --release

# ── Runtime stage ────────────────────────────────────
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/target/release/nodum /app/nodum
COPY --from=builder /app/migrations /app/migrations

# Copy frontend dist if available (for single-binary deploy)
COPY --from=builder /app/../frontend/dist /app/frontend/dist 2>/dev/null || true

RUN mkdir -p /data/uploads

EXPOSE 8080
CMD ["/app/nodum"]
