FROM oven/bun:1.4-debian AS deps
WORKDIR /app
COPY package.json bun.lock turbo.json ./
COPY apps/web/package.json ./apps/web/
COPY packages/api-client/package.json ./packages/api-client/
RUN bun install --frozen-lockfile

FROM deps AS build
WORKDIR /app
# Railway / CI may inject the git SHA so the service worker CACHE_NAME changes
# every deploy (clients pick up a new worker instead of a stale shell cache).
ARG RAILWAY_GIT_COMMIT_SHA
ARG SOURCE_COMMIT
ARG GITHUB_SHA
ENV RAILWAY_GIT_COMMIT_SHA=$RAILWAY_GIT_COMMIT_SHA \
    SOURCE_COMMIT=$SOURCE_COMMIT \
    GITHUB_SHA=$GITHUB_SHA
COPY apps/web ./apps/web
COPY packages/api-client ./packages/api-client
RUN bun run --filter @oxidean/web build

FROM oven/bun:1.4-debian AS runner
WORKDIR /app
RUN apt-get update \
  && apt-get install -y --no-install-recommends curl \
  && rm -rf /var/lib/apt/lists/*
COPY --from=build /app/package.json /app/bun.lock /app/turbo.json ./
COPY --from=build /app/node_modules ./node_modules
COPY --from=build /app/apps/web ./apps/web
COPY --from=build /app/packages/api-client ./packages/api-client
WORKDIR /app/apps/web
ENV PORT=3000
EXPOSE 3000
# Railway injects PORT at runtime; preview script honors ${PORT:-3000}.
# Platform healthcheck is GET / (no custom headers). Container HEALTHCHECK
# still uses the header-gated /health probe for local Compose.
HEALTHCHECK --interval=5s --timeout=5s --retries=12 --start-period=10s \
  CMD curl -fsS -H 'Oxidean-Health-Probe: 1' http://127.0.0.1:${PORT:-3000}/health || exit 1
CMD ["bun", "run", "preview"]
