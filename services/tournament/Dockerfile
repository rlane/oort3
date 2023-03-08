FROM rust:1.67.1-slim-buster as builder
WORKDIR /usr/src/app
RUN apt-get update
COPY . .
ARG OORT_CODE_ENCRYPTION_SECRET
RUN \
  --mount=type=cache,target=/usr/local/cargo/registry \
  --mount=type=cache,target=/usr/local/cargo/git \
  --mount=type=cache,target=/usr/src/app/services/target,sharing=locked \
  cd services/tournament && cargo install --locked --path .

FROM rust:1.67.1-slim-buster
RUN useradd -m app
USER app:1000
WORKDIR /home/app
COPY --from=builder /usr/local/cargo/bin/oort_tournament_service /usr/local/bin/oort_tournament_service
ENV PORT 8080
ENV RUST_LOG none,oort_tournament_service=info
ENV ENVIRONMENT prod
CMD ["oort_tournament_service"]
