FROM rust as build
RUN mkdir -p /app
COPY runtime  /app/runtime
COPY node /app/node
COPY pallets /app/pallets
COPY shell.nix .envrc rustfmt.toml Cargo.toml /app/
WORKDIR /app
RUN rustup default stable
RUN rustup update nightly
RUN rustup update stable
RUN rustup target add wasm32-unknown-unknown --toolchain nightly
RUN cargo build --release
