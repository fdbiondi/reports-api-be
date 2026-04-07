FROM rust:1.85-bookworm AS base

WORKDIR /usr/src/myapp

RUN apt-get update && \
    apt-get install -y --no-install-recommends sqlite3 libsqlite3-dev ca-certificates && \
    rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock ./
RUN mkdir -p src
RUN printf 'fn main() {}\n' > src/main.rs
RUN cargo build --release

FROM base AS builder

COPY src ./src
COPY data ./data

RUN cargo build --release

FROM base AS dev

RUN cargo install cargo-watch

COPY src ./src
COPY data ./data

EXPOSE 8080

CMD ["cargo", "watch", "-c", "-w", "src", "-x", "run"]

FROM debian:bookworm-slim AS runtime

WORKDIR /usr/src/myapp

RUN apt-get update && \
    apt-get install -y --no-install-recommends sqlite3 libsqlite3-0 ca-certificates && \
    rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/src/myapp/target/release/test-rust-reports-api /usr/local/bin/test-rust-reports-api
COPY --from=builder /usr/src/myapp/data ./data

EXPOSE 8080

CMD ["test-rust-reports-api"]
