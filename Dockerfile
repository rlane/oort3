FROM rust:1.71.0-slim-bookworm
WORKDIR /home
RUN apt-get update && \
    apt-get upgrade -y && \
    apt-get install -y git g++
RUN cargo install trunk wasm-opt
RUN apt install -y pkg-config
RUN apt install -y openssl
RUN apt install -y libssl-dev
RUN git clone https://github.com/rlane/oort3.git
WORKDIR /home/oort3
RUN cargo oort-serve --build-only
EXPOSE 8080
CMD cargo oort-serve --listen