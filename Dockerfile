# NetSentinel - Multi-stage Dockerfile for embedded Debian deployment
# Builds: capture (Rust), aggregator (Rust), api (Python), web (Vue.js)

# ============================================================
# Stage 1: Rust Builder
# ============================================================
FROM rust:latest AS rust-builder

RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /build

# Build capture module
COPY capture/Cargo.toml capture/Cargo.toml
COPY capture/src capture/src
RUN cd capture && cargo build --release

# Build aggregator module
COPY aggregator/Cargo.toml aggregator/Cargo.toml
COPY aggregator/src aggregator/src
RUN cd aggregator && cargo build --release

# ============================================================
# Stage 2: Node.js Builder (Frontend)
# ============================================================
FROM node:20-slim AS web-builder

WORKDIR /build/web

COPY web/package.json web/package-lock.json* ./
RUN npm install --legacy-peer-deps

COPY web/ ./
RUN npm run build

# ============================================================
# Stage 3: Capture Runtime (Rust binary with network capabilities)
# ============================================================
FROM debian:bookworm-slim AS capture

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=rust-builder /build/capture/target/release/netsentinel-capture /usr/local/bin/
COPY config/capture.docker.toml /etc/netsentinel/capture.toml

# Network capture requires elevated privileges
# Run with: --cap-add=NET_ADMIN --cap-add=NET_RAW --network=host

ENTRYPOINT ["/usr/local/bin/netsentinel-capture"]
CMD ["-c", "/etc/netsentinel/capture.toml"]

# ============================================================
# Stage 4: Aggregator Runtime (Rust binary)
# ============================================================
FROM debian:bookworm-slim AS aggregator

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=rust-builder /build/aggregator/target/release/netsentinel-aggregator /usr/local/bin/
COPY config/aggregator.docker.toml /etc/netsentinel/aggregator.toml

ENTRYPOINT ["/usr/local/bin/netsentinel-aggregator"]
CMD ["-c", "/etc/netsentinel/aggregator.toml"]

# ============================================================
# Stage 5: API Runtime (Python FastAPI)
# ============================================================
FROM python:3.11-slim-bookworm AS api

WORKDIR /app

RUN apt-get update && apt-get install -y \
    libpq5 \
    && rm -rf /var/lib/apt/lists/*

COPY api/requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt

COPY api/app ./app

EXPOSE 8000

CMD ["uvicorn", "app.main:app", "--host", "0.0.0.0", "--port", "8000"]

# ============================================================
# Stage 6: Web Runtime (Nginx serving Vue.js)
# ============================================================
FROM nginx:alpine AS web

# Remove default nginx config
RUN rm /etc/nginx/conf.d/default.conf

# Copy custom nginx config
COPY docker/nginx.conf /etc/nginx/conf.d/netsentinel.conf

# Copy built frontend
COPY --from=web-builder /build/web/dist /usr/share/nginx/html

EXPOSE 80

CMD ["nginx", "-g", "daemon off;"]
