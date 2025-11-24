FROM pihole/pihole

# Add Keepalived
RUN apk update && \
    apk add keepalived && \
    mkdir -p /etc/keepalived

COPY entrypoint.sh /entrypoint.sh

# Add Unbound
RUN apk add unbound

ENTRYPOINT ["/entrypoint.sh"]
