FROM rust:1.93-alpine AS build

WORKDIR /workspace
COPY . .

WORKDIR /workspace
RUN cargo build --release
RUN mkdir -p /workspace/static-out \
  && cp -r target/release/build/txtdot-*/out/static /workspace/static-out/static

FROM alpine:latest

WORKDIR /app
COPY --from=build /workspace/target/release/txtdot /app/txtdot
COPY --from=build /workspace/templates /app/templates
COPY --from=build /workspace/static-out/static /app/static

ENV HOST=0.0.0.0
ENV PORT=8080
ENV TXTDOT_STATIC_DIR=/app/static

EXPOSE 8080
CMD ["/app/txtdot"]
