
FROM rust:trixie AS rust_builder

WORKDIR /backend

RUN apt-get update && apt-get install -y \
  pkg-config \
  && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release

FROM debian:trixie

WORKDIR /backend

RUN apt-get update && apt-get install -y \
  ca-certificates \
  && rm -rf /var/lib/apt/lists/*

COPY --from=rust_builder /backend/target/release/server /backend/rust-server
# COPY --from=flutter_builder /frontend/build/web /frontend

EXPOSE 80

CMD ["/backend/rust-server"]
