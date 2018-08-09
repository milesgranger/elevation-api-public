#!/usr/bin/env bash

echo "Building builder image..."
docker build . -t milesg/elevation-api-builder:latest --file ./Dockerfile-Builder
echo "Build application builder, compiling binary..."
docker run --rm -v $(pwd):/build/ milesg/elevation-api-builder:latest
echo "Completed building binary at $(pwd)/build/"

echo "Building server"
docker build . -t milesg/elevation-api-server:latest --file ./Dockerfile-Server
echo "Built server, run with : 'docker run --rm -d -p 8000:8000 -v $(pwd)/processed_netcdf_files/:/data/ milesg/elevation-api-server:latest'"
