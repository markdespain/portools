#!/bin/sh
docker run --rm --name portools-service       --network portools-net -d -p 8080:8080    portools-service
