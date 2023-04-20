#!/bin/sh
./scripts/docker/create_network_bridge.sh
./scripts/docker/run_mongo_container.sh
docker run --name portools-service       --network portools-net -d -p 8080:8080    portools-service
