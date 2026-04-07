FROM rust:latest

WORKDIR /usr/src/myapp
COPY . .

RUN apt-get update && \
    apt-get install -y --no-install-recommends sqlite3 && \
    rm -rf /var/lib/apt/lists/*

RUN cargo install --path .
RUN cargo install cargo-watch

EXPOSE 8080

CMD ["test-rust-reports-api"]
