FROM rust:1.61 as builder
WORKDIR /usr/src/app
COPY . .
RUN \
  --mount=type=cache,target=/usr/local/cargo/registry \
  --mount=type=cache,target=/usr/local/cargo/git \
  --mount=type=cache,target=/usr/src/app/target \
  cd server && cargo install --locked --path .

FROM rust:1.61-slim-buster
COPY --from=builder /usr/local/cargo/bin/oort_server /usr/local/bin/oort_server
ENV PORT 8080
CMD ["oort_server"]
