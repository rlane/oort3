# fetcher
FROM rust:1.64-slim-buster as fetcher
WORKDIR /usr/src/app
RUN cargo init --bin dummy
COPY Cargo.lock.user Cargo.lock
RUN \
  --mount=type=cache,target=/usr/local/cargo/registry \
  --mount=type=cache,target=/usr/local/cargo/git \
  --mount=type=cache,target=/usr/src/app/services/target \
  cd dummy && cargo fetch && cp -a /usr/local/cargo/registry /registry

# builder
FROM rust:1.64-slim-buster as builder
WORKDIR /usr/src/app
RUN mkdir -p services
COPY . .
RUN \
  --mount=type=cache,target=/usr/local/cargo/registry \
  --mount=type=cache,target=/usr/local/cargo/git \
  --mount=type=cache,target=/usr/src/app/services/target \
  cd services/compiler && cargo install --locked --path . && cp -a /usr/local/cargo/registry /registry

# service
FROM rust:1.64-slim-buster
RUN rustup target add wasm32-unknown-unknown
WORKDIR /usr/src/app
COPY Cargo.toml.user Cargo.toml
COPY Cargo.lock.user Cargo.lock
COPY --from=fetcher /registry /usr/local/cargo/registry
COPY scripts/build-ai.sh scripts/build-ai-fast.sh scripts/
COPY --from=builder /usr/local/cargo/bin/oort_compiler_service /usr/local/bin/oort_compiler_service
COPY shared/ai ai
COPY shared/api api
RUN cargo fetch
RUN ./scripts/build-ai.sh
ENV PORT 8080
ENV RUST_LOG info
CMD ["oort_compiler_service"]
