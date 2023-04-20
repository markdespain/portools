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

FROM builder as service_builder
RUN cargo build --bin portools-service

FROM builder as stream_builder
RUN cargo build --bin portools-stream

FROM debian:bullseye-slim as runtime
ENV MONGODB_URI="mongodb://portools-mongo:27017"
RUN apt-get update && apt-get install -y curl iputils-ping && rm -rf /var/lib/apt/lists/*

FROM runtime as service_runtime
COPY --from=service_builder /portools/target/debug/portools-service /usr/local/bin/portools-service
CMD ["portools-service"]

FROM runtime as stream_runtime
COPY --from=stream_builder /portools/target/debug/portools-stream /usr/local/bin/portools-stream
CMD ["portools-stream"]