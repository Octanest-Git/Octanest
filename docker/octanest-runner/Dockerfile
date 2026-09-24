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
# curl: fetch the docker CLI static binary below.
RUN apt-get update \
  && apt-get install -y --no-install-recommends ca-certificates curl git \
  && rm -rf /var/lib/apt/lists/*
# docker CLI only (no daemon) — enables `label:docker://image` execution when
# /var/run/docker.sock is mounted. Host-mode labels need no Docker. Fetched as
# a static binary: Railway's builder rejects `COPY --from=<external image>`.
ARG DOCKER_CLI_VERSION=28.5.2
RUN curl -fsSL "https://download.docker.com/linux/static/stable/x86_64/docker-${DOCKER_CLI_VERSION}.tgz" \
    | tar -xz --strip-components=1 -C /usr/local/bin docker/docker
COPY --from=builder /app/target/release/octanest-runner /usr/local/bin/octanest-runner

# Default labels: host execution so jobs run without a Docker daemon.
# Add `name:docker://image` labels for containerized steps (requires the
# Docker socket mount).
ENV OCTANEST_RUNNER_LABELS="ubuntu-latest,self-hosted" \
    OCTANEST_RUNNER_STATE=/data/runner.json \
    OCTANEST_RUNNER_WORK_DIR=/data/work

CMD ["octanest-runner"]
