#!/bin/bash

# Build the image
docker build -t jirascope .

# Create a container from the image
id=$(docker create jirascope:latest)

# Copy the binary from the container
docker cp $id:/usr/src/app/target/release/libjirascope_dyn.so ./jirascope-dyn.so

# Remove the container
docker rm -v $id
