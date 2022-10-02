FROM rust:1.64-slim-buster as builder
WORKDIR /usr/src/app
RUN apt-get update
COPY . .
RUN \
  --mount=type=cache,target=/usr/local/cargo/registry \
  --mount=type=cache,target=/usr/local/cargo/git \
  --mount=type=cache,target=/usr/src/app/services/target \
  cd services/telemetry && cargo install --locked --path .

FROM rust:1.64-slim-buster
WORKDIR /usr/src/app
COPY --from=builder /usr/local/cargo/bin/oort_telemetry_service /usr/local/bin/oort_telemetry_service
ENV PORT 8080
ENV RUST_LOG info
CMD ["oort_telemetry_service"]
