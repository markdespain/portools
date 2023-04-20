FROM rust:1.68.2 as builder
WORKDIR /usr/src/portools
COPY . .
RUN cd ./crates/portools-service; cargo install --path .

FROM debian:bullseye-slim
RUN apt-get update && apt-get install -y curl iputils-ping && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/portools-service /usr/local/bin/portools-service
ENV MONGODB_URI="mongodb://portools-mongo:27017"
CMD ["portools-service"]