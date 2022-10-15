FROM rust:1.64-slim-buster as builder
WORKDIR /usr/src/app
RUN apt-get update
COPY . .
RUN \
  --mount=type=cache,target=/usr/local/cargo/registry \
  --mount=type=cache,target=/usr/local/cargo/git \
  --mount=type=cache,target=/usr/src/app/services/target \
  cd services/leaderboard && cargo install --locked --path .

FROM rust:1.64-slim-buster
RUN useradd -m app
USER app:1000
WORKDIR /home/app
COPY --from=builder /usr/local/cargo/bin/oort_leaderboard_service /usr/local/bin/oort_leaderboard_service
ENV PORT 8080
ENV RUST_LOG info
ENV ENVIRONMENT prod
CMD ["oort_leaderboard_service"]
