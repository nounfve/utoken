FROM ubuntu:22.04
COPY ./utoken /xbin/

WORKDIR /app/
ENV PATH="/xbin:${PATH}"
ENTRYPOINT [ "utoken" ]
