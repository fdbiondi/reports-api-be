### Create rust project

```bash
cargo init --bin ProjectName
```

### Build the project

```bash
cargo build
```

### Run compilation

```bash
cargo run
```

#### Run with docker and expose port

```bash
docker run --rm -v "${PWD}/reports-api:/app" -w "/app" -p 8080:80 --name rust-test -it rust:latest bash -c "cargo build && cargo run"
```

#### Run rust container using docker compose

```bash
docker-compose run --rm rust_dev
```
