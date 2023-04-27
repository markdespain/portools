#!/bin/sh
set -e
docker run --rm --name portools-mongo --hostname portools-mongo --network portools-net -d -p 27017:27017 mongo
