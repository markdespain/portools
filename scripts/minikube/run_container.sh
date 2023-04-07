#!/bin/sh
# based on https://auth0.com/blog/kubernetes-tutorial-step-by-step-introduction-to-basic-concepts/

echo "create deployment"
kubectl apply -f minikube/deployment.yaml

echo "enable ingress addon"
minikube addons enable ingress
minikube addons enable ingress-dns
kubectl get pods -n ingress-nginx

echo "add service"
kubectl apply -f minikube/service.yaml

echo "add ingress"
kubectl apply -f minikube/ingress.yaml

echo "show ingress ip"
kubectl get ingress portools-ingress -o=jsonpath='{.status.loadBalancer.ingress[0].ip}'
