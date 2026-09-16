FROM oven/bun:1.4-debian AS deps
WORKDIR /app
COPY package.json bun.lock turbo.json ./
COPY apps/web/package.json ./apps/web/
COPY packages/api-client/package.json ./packages/api-client/
RUN bun install --frozen-lockfile

FROM deps AS build
WORKDIR /app
COPY apps/web ./apps/web
COPY packages/api-client ./packages/api-client
RUN bun run --filter @octanest/web build

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
# Header-gated probe (see vite-plugins/web-health.ts). Mirrors API /health cadence.
HEALTHCHECK --interval=5s --timeout=5s --retries=12 --start-period=10s \
  CMD curl -fsS -H 'Octanest-Health-Probe: 1' http://127.0.0.1:3000/health || exit 1
CMD ["bun", "run", "preview"]
