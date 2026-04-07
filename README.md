# Reports API

API HTTP escrita en Rust con `actix-web` para registrar reportes y mantener un contador `nonce` asociado a una `signature`.

## Resumen

La API expone dos recursos principales:

- `reports`: almacena un reporte identificado por `signature`.
- `nonces`: guarda un contador incremental por `signature`.

El flujo principal es:

1. Un cliente crea un reporte con `POST /reports`.
2. Si la `signature` no tiene nonce, la API crea uno con valor `1`.
3. Si ya existe, incrementa el nonce.
4. Luego se puede consultar el reporte o el nonce por `signature`.

## Stack técnico

- Rust 2021
- `actix-web`
- `serde`
- SQLite local (`data/data.db`)
- Docker / Docker Compose para desarrollo

## Estructura del proyecto

- `src/main.rs`: arranque del servidor HTTP.
- `src/api/nonces.rs`: endpoint de consulta de nonce.
- `src/api/reports.rs`: endpoints de consulta y creación de reportes.
- `src/model/report.rs`: acceso a datos de `reports`.
- `src/model/nonce.rs`: acceso a datos de `nonces`.
- `data/data.db`: base SQLite incluida en el repo.
- `data/README.md`: esquema SQL de referencia.

## Endpoints

### `GET /nonces/{signature}`

Busca el nonce asociado a una `signature`.

Respuesta exitosa:

```json
{
  "uuid": "3c7f2d7c-2a57-4e53-a1f2-5d6e01234567",
  "signature": "wallet-signature",
  "nonce": 2
}
```

### `GET /reports/{signature}`

Busca un reporte por `signature`.

Respuesta exitosa:

```json
{
  "uuid": "8f3fd0de-8b54-4c3c-a53a-1234567890ab",
  "signature": "wallet-signature",
  "description": "Report description",
  "title": "Report title",
  "state": "InProgress"
}
```

### `POST /reports`

Crea un reporte y actualiza el nonce asociado a la misma `signature`.

Body esperado:

```json
{
  "signature": "wallet-signature",
  "title": "Report title",
  "description": "Report description"
}
```

La `signature` se usa tanto para crear el reporte como para buscar o crear el nonce asociado.

Respuesta exitosa:

```json
{
  "uuid": "3c7f2d7c-2a57-4e53-a1f2-5d6e01234567",
  "signature": "wallet-signature",
  "nonce": 1
}
```

## Cómo ejecutar localmente

### Requisitos

- Rust toolchain instalado (`cargo`, `rustc`)
- SQLite3 instalado

### Base de datos

El proyecto ya incluye una base SQLite en `data/data.db`.

Para inspeccionar el esquema:

```bash
sqlite3 data/data.db ".schema"
```

Si necesitás recrear las tablas:

```sql
CREATE TABLE reports (
  uuid NVARCHAR(36) UNIQUE NOT NULL,
  signature NVARCHAR(132) PRIMARY KEY NOT NULL,
  description TEXT NOT NULL,
  title NVARCHAR(50) NOT NULL,
  state NVARCHAR(12) NOT NULL
);

CREATE TABLE nonces (
  uuid NVARCHAR(36) UNIQUE NOT NULL,
  signature NVARCHAR(132) PRIMARY KEY NOT NULL,
  nonce INTEGER NOT NULL
);
```

### Ejecutar con Cargo

```bash
cargo run
```

El servidor usa estas variables de entorno:

- `HOST`: por defecto `0.0.0.0`
- `PORT`: por defecto `8080`
- `DB_PATH`: por defecto `data/data.db`

Importante:

- La app carga variables desde un archivo `.env` usando `dotenv`.
- El valor por defecto `8080` evita requerir privilegios elevados en la mayoría de entornos locales.

Ejemplos:

```bash
cargo run
```

```bash
HOST=127.0.0.1 PORT=8080 DB_PATH=data/data.db cargo run
```

Archivo `.env` de ejemplo:

```env
HOST=0.0.0.0
PORT=8080
DB_PATH=data/data.db
```

Podés tomar como referencia [`.env.example`](/home/fdbiondi/dev/projects/reports-api-be/.env.example).

## Cómo compilar

```bash
cargo build
```

Para compilación optimizada:

```bash
cargo build --release
```

## Cómo testear

```bash
cargo test
```

Estado actual del proyecto:

- Hay tests automatizados básicos de endpoints usando `actix_web::test`.
- `cargo test` valida compilación y cubre `GET /reports/{signature}`, `GET /nonces/{signature}` y `POST /reports`.

## Cómo ejecutar con Docker

El `Dockerfile` define dos flujos:

- `dev`: instala `cargo-watch` y está pensado para desarrollo local
- `runtime`: imagen final más chica, pensada para ejecución o despliegue

### Docker para desarrollo

```bash
docker compose up --build
```

o, si tu instalación usa el comando viejo:

```bash
docker-compose up --build
```

Este flujo usa el target `dev` del `Dockerfile`.

La configuración actual de desarrollo:

- monta el repo en `/usr/src/myapp`
- expone `8080` del host hacia `8080` del contenedor
- usa el target `dev` del `Dockerfile`
- arranca con `cargo watch -c -w src -x run`
- usa `.dockerignore` para no enviar `target/`, `.git/` y archivos locales innecesarios al contexto de build
- construye en capas copiando primero `Cargo.toml` y `Cargo.lock` para reutilizar mejor la cache de dependencias

Eso deja la API accesible en:

```text
http://localhost:8080
```

Cuándo usarlo:

- cuando querés desarrollo con recarga automática
- cuando vas a editar código localmente mientras el contenedor corre

### Docker para runtime

```bash
docker build -t reports-api .
docker run --rm -p 8080:8080 reports-api
```

Este flujo usa el target final `runtime` del `Dockerfile`.

La imagen resultante:

- no instala `cargo-watch`
- copia solo el binario compilado y la carpeta `data/`
- está pensada para correr la API, no para editar el código dentro del contenedor

Cuándo usarlo:

- cuando querés probar la imagen final
- cuando buscás un contenedor más chico y más cercano a despliegue

## Ejemplos de uso

### Crear reporte

```bash
curl -X POST http://localhost:8080/reports \
  -H "Content-Type: application/json" \
  -d '{
    "signature": "wallet-signature",
    "title": "Broken report",
    "description": "The generated file is empty"
  }'
```

### Consultar reporte

```bash
curl http://localhost:8080/reports/wallet-signature
```

### Consultar nonce

```bash
curl http://localhost:8080/nonces/wallet-signature
```

## Observaciones sobre el estado actual

Durante la revisión aparecieron varios puntos a tener en cuenta:

- La configuración de entorno ahora se carga con `dotenv`, y la ruta de SQLite puede definirse con `DB_PATH`; si no se define, usa `data/data.db`.
- `HOST` y `PORT` ya pueden parametrizarse, y el valor por defecto de `PORT` es `8080`, lo que simplifica la ejecución local.
- `cargo test` pasa y hoy cubre los endpoints principales, pero todavía quedan warnings de compilación en los modelos.
- `POST /reports` sigue respondiendo `404 Not Found` para fallos de creación o actualización, aunque no todos esos errores representan realmente un “not found”.
- `Nonce::find` todavía ignora el resultado de `statement.bind(...)`, lo que puede ocultar errores de bind y generar diagnósticos imprecisos.

## Recomendaciones

- Corregir los códigos HTTP de error en `POST /reports` para distinguir mejor entre validaciones, conflictos y errores internos.
- Manejar explícitamente el resultado de `statement.bind(...)` en `Nonce::find`.
- Ampliar la cobertura de tests para casos de error, validaciones y regresiones de base de datos.
- Limpiar los warnings de compilación restantes en `cargo test`.
