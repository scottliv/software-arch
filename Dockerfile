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