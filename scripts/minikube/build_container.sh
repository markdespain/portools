#!/bin/sh

#based on https://medium.com/swlh/how-to-run-locally-built-docker-images-in-kubernetes-b28fbc32cc1d
eval "$(minikube -p minikube docker-env)"
./scripts/docker/build_container.sh