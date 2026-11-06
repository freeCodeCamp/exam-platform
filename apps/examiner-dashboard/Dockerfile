# syntax=docker/dockerfile:1
# Build context: repo root (see compose.yaml).

# Client: the root Bun workspace lockfile needs every member manifest to stay frozen.
FROM oven/bun:1.4.2 AS client
WORKDIR /src
COPY package.json bun.lock ./
COPY apps/web-app/package.json apps/web-app/
COPY apps/desktop-app/package.json apps/desktop-app/
COPY apps/examiner-dashboard/package.json apps/examiner-dashboard/
RUN --mount=type=cache,target=/root/.bun/install/cache \
    bun install --frozen-lockfile --filter examiner-dashboard-client
COPY apps/examiner-dashboard/astro.config.ts apps/examiner-dashboard/tsconfig.json apps/examiner-dashboard/
COPY apps/examiner-dashboard/client apps/examiner-dashboard/client
RUN bun --filter examiner-dashboard-client build

# Server: every Rust workspace member manifest must be present for cargo to load the workspace.
FROM rust:1-slim-trixie AS build
WORKDIR /src
COPY Cargo.toml Cargo.lock ./
COPY apps/auth-api apps/auth-api
COPY apps/examiner-dashboard apps/examiner-dashboard
COPY libs libs
COPY databases/curriculum databases/curriculum
COPY databases/examiner databases/examiner
# Checks `query!` macros against the committed .sqlx/, so the build needs no database.
ENV SQLX_OFFLINE=true
# `dev` for local development only (compose.dev.yaml): debug builds allow MOCK_AUTH. Never deploy one.
ARG CARGO_PROFILE=release
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/src/target \
    cargo build --locked --profile "$CARGO_PROFILE" -p examiner-dashboard -p edb -p cdb \
    && out="target/$([ "$CARGO_PROFILE" = dev ] && echo debug || echo "$CARGO_PROFILE")" \
    && cp "$out/server" /usr/local/bin/examiner-dashboard \
    && cp "$out/edb-migrate" "$out/cdb-migrate" /usr/local/bin/

FROM debian:trixie-slim
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates curl \
    && rm -rf /var/lib/apt/lists/*
RUN useradd --system --no-create-home --shell /usr/sbin/nologin app
COPY --from=build /usr/local/bin/examiner-dashboard /usr/local/bin/examiner-dashboard
# Deploy runs migrations as separate, one-shot containers (compose.yaml `migrate`, `migrate-cdb`).
COPY --from=build /usr/local/bin/edb-migrate /usr/local/bin/edb-migrate
COPY --from=build /usr/local/bin/cdb-migrate /usr/local/bin/cdb-migrate
COPY --from=client /src/apps/examiner-dashboard/dist /app/web
USER app
ENV LISTEN_ADDR=0.0.0.0:13002 RUST_LOG=info WEB_DIR=/app/web
EXPOSE 13002
HEALTHCHECK --interval=10s --timeout=5s --retries=5 CMD ["curl", "-fsS", "http://127.0.0.1:13002/healthz"]
ENTRYPOINT ["examiner-dashboard"]
