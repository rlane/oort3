FROM rust:1.68.0-slim-buster as builder
WORKDIR /usr/src/app
RUN apt-get update
COPY . .
ARG OORT_ENVELOPE_SECRET
ARG OORT_CODE_ENCRYPTION_SECRET
ARG DISCORD_LEADERBOARD_WEBHOOK
ARG DISCORD_TELEMETRY_WEBHOOK
RUN \
  --mount=type=cache,target=/usr/local/cargo/registry \
  --mount=type=cache,target=/usr/local/cargo/git \
  --mount=type=cache,target=/usr/src/app/target,id=services_target,sharing=locked \
  cd services/backend && cargo install --profile service --locked --path .

FROM rust:1.68.0-slim-buster
RUN useradd -m app
USER app:1000
WORKDIR /home/app
COPY --from=builder /usr/local/cargo/bin/oort_backend_service /usr/local/bin/oort_backend_service
ENV PORT 8080
ENV RUST_LOG none,oort_backend_service=info
CMD ["oort_backend_service"]
