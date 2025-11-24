FROM rust:1.91-alpine3.22 AS build

# Build Microstatus
COPY external/Cargo.lock \
    external/Cargo.toml \
    /microstatus/

COPY external/src /microstatus/src
COPY external/templates /microstatus/templates

WORKDIR /microstatus
RUN apk add build-base musl-dev && \
    cargo build --release

FROM alpine:3.22.2

# Setup created Microstatus binary
WORKDIR /microstatus
COPY --from=build /microstatus/target/release/microstatus /bin/microstatus

# Setup NGINX
RUN apk update && apk add nginx
RUN mkdir /etc/nginx/ssl && \
    /bin/rm -f /etc/nginx/http.d/* && \
    sed -i "s/worker_processes auto;/worker_processes 1;/" /etc/nginx/nginx.conf

# Setup Keepalived
RUN apk add keepalived
RUN mkdir -p /etc/keepalived

# Setup entrypoint
COPY entrypoint.sh /entrypoint.sh
RUN chmod +x /entrypoint.sh

ENTRYPOINT [ "/entrypoint.sh" ]