FROM lukemathwalker/cargo-chef:latest as chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare

FROM chef AS builder
COPY --from=planner /app/recipe.json .
RUN cargo chef cook --release
COPY . .
RUN cargo build --release

FROM debian:stable-slim AS rust-server
WORKDIR /app
COPY --from=builder /app/target/release/server /server
ENV APP_ENVIRONMENT production
CMD ["/server"]
LABEL service=rust-server

FROM debian:stable-slim AS image_collector
WORKDIR /app
COPY --from=builder /app/target/release/image_collector /image_collector
ENV APP_ENVIRONMENT production
CMD ["/image_collector"]
LABEL service=image_collector

FROM debian:stable-slim AS image_generator
WORKDIR /app
RUN apt-get update && apt install -y openssl
COPY --from=builder /app/target/release/image_generator /image_generator
ENV APP_ENVIRONMENT production
CMD ["/image_generator"]
LABEL service=image_generator

FROM debian:stable-slim AS migration
WORKDIR /app
COPY --from=builder /app/target/release/migration /migration
CMD ["/migration"]
LABEL service=migration
