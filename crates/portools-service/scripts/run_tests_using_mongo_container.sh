#!/bin/sh
# use a single thread in order to avoid sporadic failures due to
# concurrent tests operating on a shared database and collections
MONGODB_URI="mongodb://localhost:27017" cargo test -- --test-threads=1