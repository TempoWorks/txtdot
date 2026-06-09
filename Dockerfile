FROM rust:1.88-bookworm AS build

WORKDIR /workspace
COPY drova ./drova
COPY txtdot ./txtdot

WORKDIR /workspace/txtdot
RUN cargo build --release
RUN mkdir -p /workspace/static-out \
  && cp -r target/release/build/txtdot-*/out/static /workspace/static-out/static

FROM debian:bookworm-slim

RUN apt-get update \
  && apt-get install -y --no-install-recommends ca-certificates \
  && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=build /workspace/txtdot/target/release/txtdot /app/txtdot
COPY --from=build /workspace/txtdot/templates /app/templates
COPY --from=build /workspace/static-out/static /app/static

ENV HOST=0.0.0.0
ENV PORT=8080
ENV TXTDOT_STATIC_DIR=/app/static

EXPOSE 8080
CMD ["/app/txtdot"]
