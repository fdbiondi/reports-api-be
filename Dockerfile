FROM rust:latest

WORKDIR /usr/src/myapp
COPY . .

RUN apt-get update && apt-get upgrade -y && \
    apt-get install -y sqlite3

RUN cargo install --path .
RUN cargo install cargo-watch

EXPOSE 8080

CMD ["myapp"]
