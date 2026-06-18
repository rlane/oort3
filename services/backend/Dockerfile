FROM rust:1.96.0-slim-bookworm as builder
WORKDIR /usr/src/app
RUN apt-get update && echo 20260516
RUN apt-get install -y clang binaryen
COPY . .
ARG OORT_ENVELOPE_SECRET
ARG OORT_CODE_ENCRYPTION_SECRET
ARG DISCORD_LEADERBOARD_WEBHOOK
ARG DISCORD_TELEMETRY_WEBHOOK
RUN \
  --mount=type=cache,target=/usr/local/cargo/registry \
  --mount=type=cache,target=/usr/local/cargo/git \
  --mount=type=cache,target=/usr/src/app/target,id=services_target,sharing=locked \
  cargo install --profile service --locked --path services/backend && \
  cargo install --profile service --locked --path tools --bin tournament

FROM rust:1.96.0-slim-bookworm
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
RUN useradd -m app
USER app:1000
WORKDIR /home/app
COPY --from=builder /usr/local/cargo/bin/oort_backend_service /usr/local/bin/oort_backend_service
COPY --from=builder /usr/local/cargo/bin/tournament /usr/local/bin/tournament
ENV PORT 8080
ENV RUST_LOG none,oort_backend_service=info
CMD ["oort_backend_service", "serve"]
