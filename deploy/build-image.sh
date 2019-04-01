#!/bin/sh
set -e

# Build backend bianry
docker-compose -f docker-compose-build.yml up

mkdir -p ./tmp/build/

# Copy targets from builder container
docker container cp backend-builder:/app/target/release/elba-backend ./tmp/build/elba-backend
docker container cp backend-builder:/root/diesel ./tmp/build/diesel

# Build production image
docker build -t elba/backend:latest . -f Dockerfile.run

rm ./tmp/build/
