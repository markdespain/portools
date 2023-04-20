#!/bin/bash
set -e

if [ "$1" != "portools-service" ] && [ "$1" != "portools-stream" ]; then
  echo "argument my be 'portools-service' or 'portools-stream'"
  exit 1
fi

trap 'popd' EXIT
pushd ..
# saves time since the Docker file's COPY task will avoid copying prior build output
cargo clean
docker build -t "$1" --target "$1" .