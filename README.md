# Mini Projects

**Small pieces of code that don't deserve their own repository but I want to share** 

Some may be specific to my setup only but they can be interesting or maybe one could be useful to someone else

---

## Table of contents
1. [NS-SPAM](#ns-spam)
2. [Site-Tester](#site-tester)
3. [Server-Log-Parser](#server-log-parser)
4. [SSH-Helper](#ssh-helper)
5. [Backerupperer](#backerupperer)
6. [Shell-Config](#shell-config)

---

### NS-SPAM
[View Script](./ns-spam.py)

Test performance of low power DNS Nameservers using this tool.
Stresses the heck out of the system you run it on too

**Dependencies**:
- Python3
- nslookup (dnsutils)

**Usage**:
```bash
python3 ns-spam.py {args}
```
Options:
| Key | Description | Default |
|---|-------------|----------|
| `--nameserver` | IP address of nameserver to test | 
| `-d` `--domain` | Domain to lookup | google.com |
| `-i` `--number` | Number of lookup requests to make | 100 |
| `-r` `--random` | Randomise domain (Adds a subdomain of random characters of a length between 3-7) | False |
| `-v` `--verbos` | Adds verbose output, includes IP found and time taken per request | False |
| `-t` `--threads` | Max number of synchronous requests (+-1) | ∞ |

*Required field if default is empty*

**⚠️ Important**
**Only use NS-SPAM on websites you own or have explicit permission to test. Unauthorized load testing can cause serious issues and may be illegal.**


---

### Site-Tester
[View Script](./site-tester/site-tester.py)

Site Tester is a lightweight, no-nonsense tool designed to help you stress test your website quickly and easily. Spin it up with Docker, fire off multiple requests, and see how your site holds up under load — all without complicated setup.

**Features**

- Load test any website URL
- Follow local links for deeper testing
- Control number of requests and concurrency
- Dockerized for easy setup and portability
- Simple command-line interface  
- More features planned: detailed reporting, custom headers, scheduling, and more!

**Dependencies**
- [Docker](https://docs.docker.com/engine/install/)
    OR
- Python3  
    - pip: httpx

**Docker**
Site-Tester uses Docker to manage dependencies and make it easier to run. Follow the below steps to run
- Build the docker image (only done once per version)
    ```bash
    docker build -t site-tester ./site-tester
    ```
- Run the container
    ```
    docker run {docker-args} site-tester {python-args}
    ```
**Docker Parameters**
| Flag    | Description                        |
|---------|----------------------------------|
| `--rm` | Delete container after use         | 
| `-it`    | Enable interactive mode    |

**Running**
- To run without docker use
    ```bash
    site-tester.py {python-args}
    ```

**Python Parameters**
| Flag    | Description                        | Default |
|---------|----------------------------------|---|
| `--url` | Target URL to test               | 
| `-f`    | Follow local links on the site    | False |
| `-n`    | Number of requests to perform     | 100 |
| `-p`    | Number of concurrent workers      | 10 |
| `--type`| Type of request ["get", "post"]   | get |
| `--timeout` | Set timeout in seconds for requests | 10 |
| `--ignore-ssl` | Ignore SSL errors | False |

*Required field if default is empty*

**⚠️ Important**
**Only use Site Tester on websites you own or have explicit permission to test. Unauthorized load testing can cause serious issues and may be illegal.**

---

### Server Log Parser

Made to make fun of someone who was constantly on my Minecraft server. Check logs and see player play time. Used for *Crafty* servers.

**Dependencies**
- Python

**Pre-requisites**
- Copy the server log files
    - Will update to check remote server in future

**Usage**
```bash
python3 log_parser.py {args}
```

**Parameters**

| Flag | Description | Default |
|---|---|---|
| `-f` `--file` | Single log file to parse | |
| `-d` `--directory` | Directory containing log files to parse | |
| `--player-time` | Find time spent playing from a particular player | | 
| `--list-players` | List all players who have played on the server | |
| `--time-slot` | Time slot `{start end}` to check within. Monday-Friday | |

*File or Directory required*

---

### SSH Helper
Automate SSH setup and management tasks with this script. SSH Helper streamlines key generation, distributes public keys to multiple hosts, and helps configure bastion and proxy hosts—all with minimal manual effort.

**Features**
- Generate SSH keys
- Copy public keys to all hosts in `~/.ssh/config.d`
- Use `sshpass` for password-based key transfer **(only applies to key copying with `-k`/`--key`; Linux/macOS only)**
- Configure bastion hosts and proxy connections automatically

**Usage**
```bash
python3 ssh_helper.py [options]
```

**Options**
| Flag                | Description                                               |
|---------------------|----------------------------------------------------------|
| `-k`, `--key`       | Path to SSH key file to transfer                         |
| `-s`, `--sshpass`   | Use sshpass for password-based key transfer (only with `-k`) |
| `-b`, `--bastion`   | Transfer config to hosts with "bastion" in their name    |
| `-p`, `--proxy`     | Create proxy hosts for external SSH access               |

**Notes**
- Keys are sent only to hosts in the top-level of `config.d`
- Proxy hosts are created in `~/.ssh/config.d/proxy/`
- Verbose output and timeouts help manage offline hosts

**Dependencies**
- Python3
- sshpass (optional, only for key copying with `-k`; Linux/macOS only)

**Examples**

- Copy public key to all hosts
    ```bash
    python3 ssh_helper.py -k ~/.ssh/id_rsa.pub
    ```
- Copy public key using sshpass (password-based transfer)
    ```bash
    python3 ssh_helper.py -k ~/.ssh/id_rsa.pub -s
    ```
- Configure bastion hosts
    ```bash
    python3 ssh_helper.py -b
    ```
- Create proxy hosts
    ```bash
    python3 ssh_helper.py -p
    ```

---

### Backerupperer

Effortlessly back up multiple directories using rsync! Configure everything in a simple YAML file and run one command to synchronize exactly what you want

[View Script](./backerupperer/backup.sh) | [View YAML Config](./backerupperer/backup.yaml)

### How It Works
1. Configure which directories and files to back up in `backup.yaml`.
2. Run the script:  
   ```
   bash backerupperer/backup.sh
   ```
3. Back up only the newest files, filter by keywords, and set file type limits.

### Example YAML Configuration
```yaml
directories:
    - path: "./test_files"
      backup_location: "./test_back"
      keywords:
          - "402"
          - "301"
      file_type: .vma.gz
      type: newest
      limit: 1
    - path: "./other_files"
      backup_location: "./other_back"
      # No keywords or file_type: backs up everything in this directory
```
**YAML Options Explained:**

| Option            | Description |
|------------------|-------------|
| `keywords`        | Only backup files whose names contain these keywords (uses `grep`). |
| `file_type`       | Limit backup to specific file types (e.g., `.vma.gz`). |
| `type`            | Defines the backup order. `newest` backs up the newest files first. |
| `limit`           | How many files per keyword to back up, based on the `type` order. |

### Example Usage

Run a backup with your configuration:
```
bash backerupperer/backup.sh
```

Dry run (shows what would be copied without actually copying):
```
bash backerupperer/backup.sh --dry
```

### Script Options

| Option          | Description                   | Default   |
|-----------------|-------------------------------|-----------|
| `--dry`         | Dry run (no files copied)     | Disabled  |
| `--delete`      | Delete old file               | Disabled  |
| `--manual`      | Show progress of transfer     | Disabled  |
| `-h`, `--help`  | Show usage information        |           |

### Why Use Backerupperer?
- No more manual file selection
- Easy YAML config for flexible backups
- Ideal for log files, VM images, or any files you want to keep safe

---

### Shell-Config

Custom configs for BASH and ZSH to display different terminal lines.

**Features:**
- Git Branch
    - Displays current branch with symbol if it is not detected to be main branch
- Directory
    - Show directory without taking up whole screen. Once the directory label becomes too long it will shorten by replacing each directory in the chain with `.` to still show depth. Will always show last directory and generally show second last. 
    - Easy distinction whether you are in within your HOME `~` or root `/`
- I like how the colors look :)

[ZSH](./shell_configs/zshrc) | [BASH](./shell_configs/bashrc)

**Usage**
- Find which shell you are using 
    ```bash
    echo $SHELL
    ```
- **`zsh`**
    - **TEST IT WORKS ON YOUR SYSTEM**
        PLEASE DO THIS 
        - Temporarily change the interpreter
            ```bash
            source ./shell_config/zshrc
            ```
        - Change to a couple directories
        - Change to a directory with a git repo
    - Create backup of your current `.zshrc` 
        ```bash
        cp ~/.zshrc ~/.zshrc-bak
        ```
    - Copy the `zshrc` to your home directory and rename `.zshrc`
        ```bash
        cp ./shell_config/zshrc ~/.zshrc
        ```
        If prompted confirm to overwrite file
    - Will use this source automatically on a shell opening

- **`bash`**
    - **TEST IT WORKS ON YOUR SYSTEM**
        PLEASE DO THIS 
        - Temporarily source the config
            ```bash
            source ./shell_config/bashrc
            ```
        - Change to a couple directories
        - Change to a directory with a git repo
    - Create backup of your current `.bashrc` 
        ```bash
        cp ~/.bashrc ~/.bashrc-bak
        ```
    - Copy the `bashrc` to your home directory and rename `.bashrc`
        ```bash
        cp ./shell_config/bashrc ~/.bashrc
        ```
        If prompted confirm to overwrite file
    - Will use this source automatically on a shell opening

---