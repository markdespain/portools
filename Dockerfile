FROM rust:1.68.2 as chef
RUN cargo install cargo-chef
WORKDIR /portools

FROM chef as planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef as builder
COPY --from=planner /portools/recipe.json recipe.json
# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --recipe-path recipe.json
COPY . .
RUN cargo build --bin portools-service

FROM debian:bullseye-slim as runtime
ENV MONGODB_URI="mongodb://portools-mongo:27017"
RUN apt-get update && apt-get install -y curl iputils-ping && rm -rf /var/lib/apt/lists/*
COPY --from=builder /portools/target/debug/portools-service /usr/local/bin/portools-service
CMD ["portools-service"]