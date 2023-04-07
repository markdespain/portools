#!/bin/sh
docker network create --driver bridge portools-net
:
docker run --rm --name portools-mongo --network portools-net -d              mongo
docker run --rm --name portools       --network portools-net -d -p 8080:8080 portools
