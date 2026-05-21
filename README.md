[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)

# Sectxt
Sending secret messages using a password.

## Crates
- [sectxt-core](crates/sectxt-core) - domain layer
- [sectxt-db](crates/sectxt-db) - database layer
- [sectxt-lambda-message](crates/sectxt-lambda-message) - AWS lambda function for handling message uploads
- [sectxt-shared](crates/sectxt-shared) - common code (such as DTOs)

## Development
### Setup
```shell
docker compose up -d
sqlx migrate run --source 'crates/sectxt-db/migrations'
cargo run
```

### Running Tests
```shell
docker compose up -d
cargo test
```

## Production
### Deployment
```shell
aws login
cargo lambda build --arm64 --release --output-format zip
terraform init
terraform apply -auto-approve
sqlx migrate run --source 'crates/sectxt-db/migrations' --database-url "$(terraform output -raw database_url)"
```

### Test Endpoint
Test create message:
```shell
curl -X POST -H "Content-Type: application/json" "$(terraform output -raw create_endpoint_url)" -d '{
  "auth_token": [1,2,3,4,5,6,7,8,9,0,1,2,3,4,5,6,7,8,9,0,1,2,3,4,5,6,7,8,9,0,1,2],
  "ciphertext": [1,2,3,4,5,6,7,8,9,1,2,3,4,5,6,7,8,9,1,2,3,4,5,6,7,8,9],
  "nonce": [1,2,3,4,5,6,7,8,9,0,1,2],
  "salt": [1,2,3,4,5,6,7,8,9,0,1,2,3,4,5,6]
}'
```

Test consume message:
```shell
curl -X POST -H "Content-Type: application/json" "$(terraform output -raw consume_endpoint_url)" -d '{
  "auth_token": [1,2,3,4,5,6,7,8,9,0,1,2,3,4,5,6,7,8,9,0,1,2,3,4,5,6,7,8,9,0,1,2],
  "id": "<ID>"
}'
```

### Destroy
```shell
terraform destroy -auto-approve
```

## TODO
- [ ] Finish AWS lambda function and hook up Neon DB
- [ ] Add file upload support
- [ ] Implement frontend
