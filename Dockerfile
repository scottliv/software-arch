FROM lukemathwalker/cargo-chef:latest as chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --bin server

FROM chef AS builder
COPY --from=planner /app/recipe.json .
RUN cargo chef cook --release --bin server
COPY . .
RUN cargo build --release
RUN mv ./target/release/server ./app

FROM debian:stable-slim AS runtime
WORKDIR /app
COPY --from=builder /app/app /usr/local/bin/
ENTRYPOINT ["/usr/local/bin/app"]