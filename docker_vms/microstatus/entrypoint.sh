#!/bin/sh

keepalived -f /etc/keepalived/keepalived.conf & 

nginx -g 'daemon off;' & 

# Ensure the correct locations are used for html output and check yaml
cp /microstatus/.env_internal /microstatus/.env
sed -i "s|^HTML_OUTPUT_DIR=.*|HTML_OUTPUT_DIR=/microstatus/output/|" /microstatus/.env
grep -q '^CHECK_FILE=' /microstatus/.env \
    && sed -i 's|^CHECK_FILE=.*|CHECK_FILE=checks.yaml|' /microstatus/.env \
    || echo 'CHECK_FILE=checks.yaml' >> /microstatus/.env

/bin/microstatus 
