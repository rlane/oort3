FROM rust:1.64-slim-buster as builder
WORKDIR /usr/src/app
RUN apt-get update
RUN apt-get install -y musl-tools
RUN rustup target add x86_64-unknown-linux-musl
COPY . .
RUN \
  --mount=type=cache,target=/usr/local/cargo/registry \
  --mount=type=cache,target=/usr/local/cargo/git \
  --mount=type=cache,target=/usr/src/app/target \
  cd services/leaderboard && cargo install --target x86_64-unknown-linux-musl --locked --path .

FROM scratch
WORKDIR /usr/src/app
COPY --from=builder /usr/local/cargo/bin/oort_leaderboard_service /usr/local/bin/oort_leaderboard_service
ENV PORT 8080
ENV RUST_LOG info
CMD ["oort_leaderboard_service"]
