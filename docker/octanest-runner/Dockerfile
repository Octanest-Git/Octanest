# Official Octanest Actions runner.
#
# Runs the native `octanest-runner` binary against Octanest's JSON runner
# protocol (/api/actions/*). Executes `actions/checkout` via git clone and
# `run:` steps on the host or in docker:// containers.
#
# Build context is the REPOSITORY ROOT (needs workspace Cargo.toml/lock +
# crates/), matching crates/octanest-api/Dockerfile.
FROM rust:1-bookworm AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
RUN cargo build --release -p octanest-runner --bin octanest-runner

FROM debian:bookworm-slim
# git: actions/checkout clones over smart HTTP; ca-certificates: TLS origins.
RUN apt-get update \
  && apt-get install -y --no-install-recommends ca-certificates git \
  && rm -rf /var/lib/apt/lists/*
# docker CLI only (no daemon) — enables `label:docker://image` execution when
# /var/run/docker.sock is mounted. Host-mode labels need no Docker.
COPY --from=docker:27-cli /usr/local/bin/docker /usr/local/bin/docker
COPY --from=builder /app/target/release/octanest-runner /usr/local/bin/octanest-runner

LABEL org.opencontainers.image.title="octanest-runner" \
      org.opencontainers.image.description="Official Octanest Actions runner" \
      org.opencontainers.image.source="https://github.com/octanest/octanest"

# Default labels: host execution so jobs run without a Docker daemon.
# Add `name:docker://image` labels for containerized steps (requires the
# Docker socket mount).
ENV OCTANEST_RUNNER_LABELS="ubuntu-latest,self-hosted" \
    OCTANEST_RUNNER_STATE=/data/runner.json \
    OCTANEST_RUNNER_WORK_DIR=/data/work

VOLUME /data
ENTRYPOINT ["/usr/local/bin/octanest-runner"]
