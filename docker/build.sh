#!/bin/bash

trap 'build' ERR

build() {
	echo "Error in build"
	exit 1
}

docker build -t daoplays/verify_$1 --file ../docker/verify_$1.dockerfile .
rm ../docker/verify_$1.dockerfile
