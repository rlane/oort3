# fetcher
FROM rust:1.68.0-slim-buster as fetcher
WORKDIR /usr/src/app
RUN cargo init --bin dummy
COPY Cargo.lock.user Cargo.lock
RUN \
  --mount=type=cache,target=/usr/local/cargo/registry \
  --mount=type=cache,target=/usr/local/cargo/git \
  --mount=type=cache,target=/usr/src/app/target,id=services_target,sharing=locked \
  cd dummy && cargo fetch && cp -a /usr/local/cargo/registry /registry

# builder
FROM rust:1.68.0-slim-buster as builder
WORKDIR /usr/src/app
COPY . .
ARG OORT_CODE_ENCRYPTION_SECRET
RUN \
  --mount=type=cache,target=/usr/local/cargo/registry \
  --mount=type=cache,target=/usr/local/cargo/git \
  --mount=type=cache,target=/usr/src/app/target,id=services_target,sharing=locked \
  cd services/compiler && cargo install --profile service --locked --path .

# service
FROM rust:1.68.0-slim-buster
RUN rustup target add wasm32-unknown-unknown
RUN rustup component add rustfmt
RUN useradd -m app
USER app:1000
WORKDIR /home/app
COPY --from=builder /usr/local/cargo/bin/oort_compiler_service /usr/local/bin/oort_compiler_service
ENV PORT 8080
ENV RUST_LOG info
RUN oort_compiler_service --prepare
CMD ["oort_compiler_service"]
