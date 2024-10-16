# frontend

# build stage
FROM node:22-alpine AS fbase
ENV PNPM_HOME="/pnpm"
ENV PATH="$PNPM_HOME:$PATH"
RUN corepack enable
COPY ./frontend /app
WORKDIR /app

FROM fbase AS fprod-deps
RUN --mount=type=cache,id=pnpm,target=/pnpm/store pnpm install --prod --frozen-lockfile

FROM fbase AS fbuild
RUN --mount=type=cache,id=pnpm,target=/pnpm/store pnpm install --frozen-lockfile
RUN pnpm run build

# production stage
FROM ghcr.io/saitewasreset/mission_backend_rs:latest
COPY --from=fbuild /app/dist /static
CMD ["mission-backend-rs"]