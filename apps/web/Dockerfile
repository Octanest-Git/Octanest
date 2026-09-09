# syntax=docker/dockerfile:1

FROM oven/bun:1.2-debian AS deps
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

FROM oven/bun:1.2-debian AS runner
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
HEALTHCHECK --interval=10s --timeout=5s --retries=12 \
  CMD curl -fsS http://127.0.0.1:3000/ || exit 1
CMD ["bun", "run", "preview"]
