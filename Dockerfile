FROM rust:slim AS builder

WORKDIR /app
COPY Cargo.toml Cargo.lock* ./
COPY src/ src/

RUN cargo build --release --features api --bin c4-api

FROM debian:bookworm-slim

COPY --from=builder /app/target/release/c4-api /usr/local/bin/c4-api

EXPOSE 3000

CMD ["c4-api"]
