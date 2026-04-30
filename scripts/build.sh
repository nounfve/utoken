#!/bin/bash
gitRoot=$(git rev-parse --show-toplevel)

cd ${gitRoot}/scripts/

cargo build
docker compose -f build.docker-compose.yaml run --build --rm utoken