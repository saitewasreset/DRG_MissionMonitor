# build stage
FROM node:22-alpine AS base
ENV PNPM_HOME="/pnpm"
ENV PATH="$PNPM_HOME:$PATH"
RUN corepack enable
COPY . /app
WORKDIR /app

FROM base AS prod-deps
RUN --mount=type=cache,id=pnpm,target=/pnpm/store pnpm install --prod --frozen-lockfile

FROM base AS build
RUN --mount=type=cache,id=pnpm,target=/pnpm/store pnpm install --frozen-lockfile
RUN pnpm run build

# production stage
FROM nginx:stable-alpine-slim AS production-stage
COPY --from=build /app/dist /usr/share/nginx/html
COPY --from=build /app/nginx_default.conf /etc/nginx/conf.d/default.conf
CMD ["nginx", "-g", "daemon off;"]