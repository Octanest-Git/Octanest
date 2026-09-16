FROM rust:1-bookworm AS builder
WORKDIR /app
# prost-build needs `protoc` for proto/runner.proto (Phase 19 Actions protocol).
RUN apt-get update \
  && apt-get install -y --no-install-recommends protobuf-compiler \
  && rm -rf /var/lib/apt/lists/*
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
RUN cargo build --release -p octanest-api --bin octanest-api

FROM debian:bookworm-slim
# git required for forge ops + D-33 boot gate (GIT-09); ca-certificates/curl for TLS + healthcheck
RUN apt-get update \
  && apt-get install -y --no-install-recommends ca-certificates curl git \
  && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/octanest-api /usr/local/bin/octanest-api
ENV API_BIND=0.0.0.0:8080
EXPOSE 8080
HEALTHCHECK --interval=5s --timeout=5s --retries=12 \
  CMD curl -fsS http://127.0.0.1:8080/health || exit 1
CMD ["octanest-api"]
