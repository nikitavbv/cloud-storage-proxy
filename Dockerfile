FROM frolvlad/alpine-glibc:alpine-3.9_glibc-2.29

WORKDIR /app
COPY target/release/cloud-storage-proxy /app/cloud-storage-proxy

ENTRYPOINT [ "/app/cloud-storage-proxy" ]