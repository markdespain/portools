#!/bin/bash
set -e
trap 'popd' EXIT

pushd ../../..
# saves time since the Docker file's COPY task will avoid copying prior build output
cargo clean
docker build -t portools-stream --target stream_runtime .