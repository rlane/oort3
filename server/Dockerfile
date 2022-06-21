FROM rust:1.61 as builder
WORKDIR /usr/src/app
COPY . .
RUN \
  --mount=type=cache,target=/usr/local/cargo/registry \
  --mount=type=cache,target=/usr/local/cargo/git \
  --mount=type=cache,target=/usr/src/app/target \
  cd server && cargo install --locked --path . && cp -a /usr/local/cargo/registry /registry

FROM rust:1.61-slim-buster
RUN rustup target add wasm32-unknown-unknown
WORKDIR /usr/src/app
COPY Cargo.toml.user Cargo.toml
COPY Cargo.lock scripts/build-ai.sh .
COPY --from=builder /registry /usr/local/cargo/registry
COPY --from=builder /usr/local/cargo/bin/oort_server /usr/local/bin/oort_server
COPY ai ai
COPY shared shared
RUN cargo fetch
RUN ./build-ai.sh
ENV PORT 8080
CMD ["oort_server"]
