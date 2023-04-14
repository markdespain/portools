#!/bin/sh
./docker/create_network_bridge.sh
./docker/run_mongo_container.sh
docker run --rm --name portools       --network portools-net -d -p 8080:8080    portools
