#!/bin/sh
./scripts/docker/create_network_bridge.sh
docker run --rm --name portools-mongo --network portools-net -d -p 27017:27017  mongo
