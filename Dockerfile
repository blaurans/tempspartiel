# syntax=docker/dockerfile:1

FROM rust:1.98-alpine AS build
WORKDIR /src

# Cache des dépendances
COPY Cargo.toml Cargo.lock* ./
RUN mkdir -p src && echo 'fn main(){}' > src/main.rs \
 && cargo build --release 2>/dev/null || true

COPY . .
RUN cargo build --release

FROM alpine:3.22
RUN adduser -D -H -u 10001 tempspartiel \
 && mkdir -p /data && chown tempspartiel:tempspartiel /data

COPY --from=build /src/target/release/tempspartiel /usr/local/bin/tempspartiel

USER tempspartiel
ENV TEMPS_ADDR=:8080
EXPOSE 8080
VOLUME ["/data"]

HEALTHCHECK --interval=30s --timeout=3s --start-period=5s \
  CMD wget -qO- http://127.0.0.1:8080/healthz >/dev/null || exit 1

ENTRYPOINT ["tempspartiel"]