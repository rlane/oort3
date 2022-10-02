FROM rust:1.64-slim-buster as builder
WORKDIR /usr/src/app
COPY . .
RUN \
  --mount=type=cache,target=/usr/local/cargo/registry \
  --mount=type=cache,target=/usr/local/cargo/git \
  --mount=type=cache,target=/usr/src/app/target \
  cd services/compiler && cargo install --locked --path . && cp -a /usr/local/cargo/registry /registry

FROM rust:1.64-slim-buster
RUN rustup target add wasm32-unknown-unknown
WORKDIR /usr/src/app
COPY Cargo.toml.user Cargo.toml
COPY Cargo.lock.user Cargo.lock
COPY scripts/build-ai.sh scripts/build-ai-fast.sh scripts/
COPY --from=builder /registry /usr/local/cargo/registry
COPY --from=builder /usr/local/cargo/bin/oort_compiler_service /usr/local/bin/oort_compiler_service
COPY ai ai
COPY api api
RUN cargo fetch
RUN ./scripts/build-ai.sh
ENV PORT 8080
ENV RUST_LOG info
CMD ["oort_compiler_service"]
