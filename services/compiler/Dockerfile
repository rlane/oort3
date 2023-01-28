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
ARG OORT_CODE_ENCRYPTION_SECRET
RUN \
  --mount=type=cache,target=/usr/local/cargo/registry \
  --mount=type=cache,target=/usr/local/cargo/git \
  --mount=type=cache,target=/usr/src/app/services/target \
  cd services/compiler && cargo install --locked --path .

# service
FROM rust:1.64-slim-buster
RUN rustup target add wasm32-unknown-unknown
RUN rustup component add rustfmt
RUN useradd -m app
USER app:1000
WORKDIR /home/app
RUN mkdir -p .cargo
ENV CARGO_HOME /home/app/.cargo
COPY --chown=app --from=fetcher /registry .cargo/registry
COPY --from=builder /usr/local/cargo/bin/oort_compiler_service /usr/local/bin/oort_compiler_service
COPY --chown=app Cargo.toml.user Cargo.toml
COPY --chown=app Cargo.lock.user Cargo.lock
COPY --chown=app scripts/build-ai.sh scripts/build-ai-fast.sh scripts/
COPY --chown=app shared/ai ai
COPY --chown=app shared/api api
RUN cargo fetch
RUN ./scripts/build-ai.sh
ENV PORT 8080
ENV RUST_LOG info
CMD ["oort_compiler_service"]
