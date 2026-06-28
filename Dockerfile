FROM lukemathwalker/cargo-chef:latest-rust-alpine3.23 AS chef

WORKDIR /workspace

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS build
COPY --from=planner /workspace/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN cargo build --release
RUN mkdir -p /workspace/static-out \
  && cp -r target/release/build/txtdot-*/out/static /workspace/static-out/static

FROM alpine:3.23

WORKDIR /app
RUN apk add --no-cache ca-certificates \
  && addgroup -S txtdot \
  && adduser -S -G txtdot -H -D txtdot
COPY --from=build --chown=txtdot:txtdot /workspace/target/release/txtdot /app/txtdot
COPY --from=build --chown=txtdot:txtdot /workspace/templates /app/templates
COPY --from=build --chown=txtdot:txtdot /workspace/static-out/static /app/static

ENV HOST=0.0.0.0
ENV PORT=8080
ENV TXTDOT_STATIC_DIR=/app/static

EXPOSE 8080
USER txtdot
CMD ["/app/txtdot"]
