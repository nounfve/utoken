FROM ubuntu:24.04

RUN <<EOF
    set -eu
    apt update
    apt install -y curl
EOF

COPY ./utoken /xbin/

WORKDIR /app/
ENV PATH="/xbin:${PATH}"
ENTRYPOINT [ "utoken" ]
