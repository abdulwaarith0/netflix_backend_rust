# Build stage
FROM rust:1.80-bullseye AS builder

WORKDIR /netflix_backend_rust


RUN apt-get update && apt-get install -y build-essential libssl-dev pkg-config

COPY . .

RUN cargo build --release

# Final stage
FROM ubuntu:20.04

WORKDIR /netflix_backend_rust


RUN apt-get update && apt-get install -y libssl1.1

COPY --from=builder /netflix_backend_rust/target/release/netflix_backend_rust /usr/local/bin/

EXPOSE 8080

CMD ["netflix_backend_rust"]