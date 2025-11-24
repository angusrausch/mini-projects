# Docker VMs

**My custom docker images that act like virtual machines with static individual IP addresses.**
Used to have redundant machines for my Proxmox VMs

---

## Bastion

Acts as a whole machine with an IP address

### Setup

Before use you have to copy and modify a number of files as below:

- `.env.example` -> `.env`
    - Follow instructions on each line for how to fill values
- `ssh`
    - Configuration for the use SSH keys. If using pre-made keys/authorized_keys file add to this dir
    - Will auto generate if not set
- `bastion_ssh_keys`
    - Host ssh keys 
    - Will auto generate if not set

### Usage

Use startup script included in `bastion` directory. Has options `--build`, `--start`, and `--stop`.

---
## Pihole

**Custom Pihole Image with Keepalived and Unbound included**

Acts as a whole machine with an IP address and includes Keepalived to claim a second IP 

### Setup

Before use you have to copy and modify a number of files as below:

- `.env.example` -> `.env`
    - Follow instructions on each line for how to fill values
- `keepalived.conf.example` -> `keepalived.conf`
    - Edit line 8 with the shared IP address
- `pihole-etc/` (Optional)
    - If porting from another Pihole instance, copy the entire directory from `/etc/pihole/` to `pihole-etc/`
    - NOTE: If not porting this directory will be created to persist configuration
### Usage

Use startup script included in `pihole` directory. Has options `--build`, `--start`, and `--stop`.

---

## MicroStatus

**Network status viewer with Keepalived**

Acts as a whole machine with an IP address and includes Keepalived to claim a second IP

### Setup

Before use you have to copy and modify a number of files as below:

- `.env.example` -> `.env`
    - Follow instructions on each line for how to fill values
- `.env_internal.example` -> `.env`
    - Follow instructions on each line for how to fill values
- `checks.yaml.example` -> `checks.yaml`
    - Example of major types of checks used
    - Modify file to work with your hosts
- `keepalived.conf.example` -> `keepalived.conf`
    - Edit line 8 with the shared IP address
- `ssl/`
    - Add a set of SSL keys into this directory named:
        - `nethealth.key`
        - `nethealth.crt`
    - This is required for Nginx
    - Use following command to generate:
        ```bash
        openssl req -x509 -newkey rsa:4096 -sha256 -days 365 -nodes -keyout nethealth.key -out nethealth.crt -subj "/CN=nethealth"
        ```
- `nginx.conf.example`
    - If you want access from external devices (outside of local IP ranges) remove the `deny all;` on line 14

### Usage

Use startup script included in `microstatus` directory. Has options `--build`, `--start`, and `--stop`.
Build and start can be used together in a single command.

If all the above steps are completed the container should run correctly

---

## Tinystatus

### Replaced by Microstatus

**Network status viewer with Keepalived**

Acts as a whole machine with an IP address and includes Keepalived to claim a second IP

### Setup

Before use you have to copy and modify a number of files as below:

- `.env.example` -> `.env`
    - Follow instructions on each line for how to fill values
- `checks.yaml.example` -> `checks.yaml`
    - Example of major types of checks used
    - Modify file to work with your hosts
- `incidents.md.example` -> `incidents.yaml`
    - Message displayed at bottom of page
    - Can leave blank or leave a message
- `keepalived.conf.example` -> `keepalived.conf`
    - Edit line 8 with the shared IP address
- `ssl/`
    - Add a set of SSL keys into this directory named:
        - `nethealth.key`
        - `nethealth.crt`
    - This is required for Nginx
    - Use following command to generate:
        ```bash
        openssl req -x509 -newkey rsa:4096 -sha256 -days 365 -nodes -keyout nethealth.key -out nethealth.crt -subj "/CN=nethealth"
        ```
- `tinystatus.conf`
    - If you want access from external devices (outside of local IP ranges) remove the `deny all;` on line 14

### Usage

Use startup script included in `tinystatus` directory. Has options `--build`, `--start`, and `--stop`.
Build and start can be used together in a single command.

If all the above steps are completed the container should run correctly