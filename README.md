[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)

# Sectxt
Sending secret messages.

## Backend Crates
- [`sectxt-api`](backend/crates/sectxt-api/): API layer
- [`sectxt-core`](backend/crates/sectxt-core/): domain layer
- [`sectxt-db`](backend/crates/sectxt-db/): database layer

## Run Tests
Some tests use postgres to run, set `DATABASE_URL` environment variable before running.
```shell
$env:DATABASE_URL = "postgres://postgres:postgres@127.0.0.1:5432/postgres?sslmode=disable"
cargo test
```

## Backend Dev
```shell
docker compose up -d
echo 'DATABASE_URL=postgres://postgres:postgres@127.0.0.1:5432/postgres?sslmode=disable' > .env
sqlx migrate run --source 'backend/crates/sectxt-db/migrations'
cargo run --manifest-path 'backend/Cargo.toml'

# test request
curl 127.0.0.1:8080/message -X POST -H 'Content-Type: application/json' -d '{
  "burnOnRead": false,
  "hasPassword": false,
  "ciphertext": [0, 0, 0],
  "nonce": [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11],
  "salt": [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]
}'
curl 127.0.0.1:8080/message/<uuid>
```

## Frontend Dev
```shell
npm --prefix 'frontend/' install
npm --prefix 'frontend/' run dev
```

## TODO
- [ ] Frontend
- [ ] Attachment support
