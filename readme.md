## Image Collection and storage app

prerequisites:

- install rust toolchain: https://www.rust-lang.org/tools/install
- install Docker
- unsplash access key by setting up a free demo app: https://unsplash.com/developers

run db: `docker compose up`
in a separate terminal:
build project with `cargo build`
run migrations: `cargo --bin database/migration`
run image collection service (this is a scheduled job, but can be run manually) `UNSPLASH_ACCESS_KEY={your api access key} cargo --bin image_collector`

connect to db in docker: `psql postgres://postgres:postgres@localhost:5433/rust-software-arch`

- check images have been saved: `SELECT * FROM "inspiration_image";`
