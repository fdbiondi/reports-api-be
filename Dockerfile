FROM rust:latest

WORKDIR /usr/src/myapp

RUN apt-get update && \
    apt-get install -y --no-install-recommends sqlite3 && \
    rm -rf /var/lib/apt/lists/*

RUN cargo install cargo-watch

COPY Cargo.toml Cargo.lock ./
RUN mkdir -p src
RUN printf 'fn main() {}\n' > src/main.rs
RUN cargo build --release

COPY src ./src
COPY data ./data
COPY README.md ./

RUN cargo build --release

EXPOSE 8080

CMD ["./target/release/test-rust-reports-api"]
