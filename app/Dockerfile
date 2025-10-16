FROM ubuntu:24.04

RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential \
    crossbuild-essential-arm64 \
    qemu-user-static \
    && rm -rf /var/lib/apt/lists/*
