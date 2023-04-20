#!/bin/sh

# Approach: Run MongoDB as a single node replica set.  A replicaa set is needed since the
# portools-stream crate uses MongoDB Change Sreams.
# ref: https://www.mongodb.com/docs/manual/changeStreams/
docker run --rm --name portools-mongo --hostname portools-mongo --network portools-net -d -p 27017:27017 mongo \
       --replSet "portools" --bind_ip_all

echo "sleeping before initializing the replica set, so that MongoDB can first finishing initializing"
sleep 5

echo "Initializing the replica set"
docker exec portools-mongo mongosh --eval "rs.initiate()"

echo "replica set details:"
docker exec portools-mongo mongosh --eval "rs.status()"