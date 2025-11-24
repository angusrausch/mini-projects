#!/bin/bash

# Ensure correct permissions on etc files
chmod 777 -R /etc/pihole

# Run my custom keepalived script
keepalived -f /etc/keepalived/keepalived.conf >> /var/log/keepalived.log 2>&1

# Start unbound
unbound -d -c /etc/unbound/unbound.conf >> /var/log/unbound.log 2>&1 &

# Run pihole start script 
/usr/bin/start.sh
