
FROM rust:trixie AS rust_builder

WORKDIR /backend

RUN apt-get update && apt-get install -y \
  pkg-config \
  && rm -rf /var/lib/apt/lists/*

COPY server/Cargo.toml server/Cargo.lock ./
COPY server/src ./src
RUN cargo build --release

FROM node:22-bookworm-slim AS angular_builder

WORKDIR /frontend

COPY AngularFrontend/package.json AngularFrontend/package-lock.json ./
RUN npm install -g npm@11.6.2
RUN npm ci

COPY AngularFrontend/ ./
RUN npm run build -- --configuration production --output-path /frontend/build-out

FROM debian:trixie

WORKDIR /backend

RUN apt-get update && apt-get install -y \
  ca-certificates \
  && rm -rf /var/lib/apt/lists/*

COPY --from=rust_builder /backend/target/release/server /backend/rust-server
COPY --from=angular_builder /frontend/build-out/browser /frontend/

EXPOSE 80

CMD ["/backend/rust-server"]
