FROM debian:bookworm

RUN apt update -y && \
    apt install -y iputils-ping curl dnsutils openssh-server iproute2 vim && \
    mkdir /var/run/sshd

RUN adduser bastion --disabled-password --gecos "" && \
    mkdir -p /home/bastion/.ssh && \
    chown bastion:bastion /home/bastion/.ssh

RUN chmod 644 /etc/ssh/ssh_config

# Will revert if user does not input ssh dir
RUN cp -r /etc/ssh/ /etc/ssh.bak/

COPY docker_entrypoint.sh /entrypoint.sh
RUN chmod +x /entrypoint.sh

ENTRYPOINT ["/entrypoint.sh"]
