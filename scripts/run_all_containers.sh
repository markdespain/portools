#!/bin/bash
set -e
./run_mongo_container.sh
./run_portools_service_container.sh
./run_portools_streams_container.sh