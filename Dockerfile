FROM node:23-alpine AS base
ENV PNPM_HOME="/pnpm"
ENV PATH="$PNPM_HOME:$PATH"
RUN corepack enable
WORKDIR /app
COPY package.json pnpm-lock.yaml pnpm-workspace.yaml ./
COPY packages/server/package.json packages/server/package.json
COPY packages/plugins/package.json packages/plugins/package.json
COPY packages/sdk/package.json packages/sdk/package.json

FROM base AS prod-deps
RUN --mount=type=cache,id=pnpm,target=/pnpm/store pnpm install --prod --frozen-lockfile

FROM base AS build
RUN --mount=type=cache,id=pnpm,target=/pnpm/store pnpm install --frozen-lockfile
COPY . .
RUN pnpm run build

FROM node:23-alpine AS run

WORKDIR /app

COPY --from=prod-deps /app/node_modules /app/node_modules

COPY --from=prod-deps /app/packages/server/node_modules /app/packages/server/node_modules
COPY --from=prod-deps /app/packages/plugins/node_modules /app/packages/plugins/node_modules
COPY --from=prod-deps /app/packages/sdk/node_modules /app/packages/sdk/node_modules

COPY --from=build /app/packages/sdk/dist/ /app/packages/sdk/dist
COPY --from=build /app/packages/plugins/dist/ /app/packages/plugins/dist
COPY --from=build /app/packages/server/dist/ /app/packages/server/dist

COPY --from=build /app/packages/sdk/package.json /app/packages/sdk
COPY --from=build /app/packages/plugins/package.json /app/packages/plugins
COPY --from=build /app/packages/server/package.json /app/packages/server

EXPOSE 8080
WORKDIR /app/packages/server/dist
CMD [ "node", "./src/app.js" ]
