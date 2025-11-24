#!/bin/bash
set -e

# If keys are not set generate and create authorized keys file
if [ -z "$(ls -A '/home/bastion/.ssh/')" ]; then
    echo "Creating ssh keys"
    touch /home/bastion/.ssh/authorized_keys
    ssh-keygen -t rsa -f /home/bastion/.ssh/id_rsa -N ""
fi

# Fix ownership and permissions
chown -R bastion:bastion /home/bastion/.ssh
chmod 700 /home/bastion/.ssh
chmod 600 /home/bastion/.ssh/authorized_keys

# If keys are not set default to original
if [ -z "$(ls -A '/etc/ssh/')" ]; then
    echo "Creating ssh credentials"
    /bin/cp -rf /etc/ssh.bak/* /etc/ssh/
fi

# Run the SSH daemon
exec /usr/sbin/sshd -D

