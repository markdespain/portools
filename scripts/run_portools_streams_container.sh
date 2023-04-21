#!/bin/sh
set -e
docker run --rm --name portools-stream --network portools-net -d portools-stream
