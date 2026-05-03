#!/bin/bash
gitRoot=$(git rev-parse --show-toplevel)

cd ${gitRoot}/ui_web/
npm run build

cd ${gitRoot}/scripts/
cargo build
docker compose -f build.docker-compose.yaml run --build --rm utoken