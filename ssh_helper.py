import os
import re
import shutil
from argparse import ArgumentParser
import subprocess
from pprint import pprint
from getpass import getpass
import traceback
from pathlib import Path

RED     = "\033[31m"
GREEN   = "\033[32m"
YELLOW  = "\033[33m"
BLUE    = "\033[34m"
MAGENTA = "\033[35m"
CYAN    = "\033[36m"
RESET   = "\033[0m"
BOLD    = "\033[1m"
ORANGE  = "\033[38;5;202m"

ascii_banner = fr"""
    {CYAN}
         _____ _____ _    _        _    _ ______ _      _____  ______ _____  
        / ____/ ____| |  | |      | |  | |  ____| |    |  __ \|  ____|  __ \ 
       | (___| (___ | |__| |______| |__| | |__  | |    | |__) | |__  | |__) |
        \___ \\___ \|  __  |______|  __  |  __| | |    |  ___/|  __| |  _  / 
        ____) |___) | |  | |      | |  | | |____| |____| |    | |____| | \ \ 
       |_____/_____/|_|  |_|      |_|  |_|______|______|_|    |______|_|  \_\
                                                                       {RESET}
"""

def main(args):
    config_path = Path("~/.ssh/config.d").expanduser()
    print(ascii_banner)

    if args.proxy:
        create_proxies(config_path)
    if args.bastion:
        send_hosts(config_path)
    if args.key:
        add_keys(config_path, args)

def add_keys(config_path, args, timeout = 5):
    key_path = os.path.abspath(args.key)
    sshpass = args.sshpass
    if sshpass:
        check_sshpass = subprocess.run(["which", "sshpass"], capture_output=True, text=True)
        check_sshpass_string = str(check_sshpass)
        if check_sshpass_string == "" or check_sshpass_string == "sshpass not found" or check_sshpass.returncode != 0:
            print(f"{RED}sshpass not found on system. Please install it or remove {BLUE}-s{RED} argument{RESET}")
            exit()
    hosts = get_hosts(config_path)
    print(f"{YELLOW}Sending keys to Hosts. This may take a while, current timeout is set to {BLUE}{timeout}{YELLOW} Seconds per Host{RESET}")
    print(f"{YELLOW}If you would like to skip a host use \"{BLUE}CTRL-C{YELLOW}\"{RESET}")
    print("=" * 80)
    send_keys_to_hosts(hosts, key_path, sshpass, timeout)

def send_keys_to_hosts(hosts, key_path, sshpass, timeout):
    password = get_sshpass_password() if sshpass else None
    for host in hosts:
        host_short = re.search(r"(?i)Host\s+(\S+)", host)
        host_short = host_short.group(1) if host_short else ''
        print(f"{YELLOW}Sending key to Host: {BLUE}{host_short}{RESET}")
        copy_id_command = ["ssh-copy-id", "-i", key_path, host_short]

        if sshpass:
            copy_id_command = ["sshpass", "-p", password] + copy_id_command
        try:
            result = subprocess.run(copy_id_command, timeout=timeout, capture_output=True, text=True)
        except subprocess.TimeoutExpired:
            print(f"{RED}Failed to upload Key: Timeout{RESET}")
        except KeyboardInterrupt:
            print(f"\n{ORANGE}Host skipped - User request{RESET}")
        else:
            result_string = str(result)
            if "WARNING: All keys were skipped because they already exist on the remote system." in result_string:
                print(f"{GREEN}Key already on host{RESET}")
            elif "Permission denied (publickey)." in result_string:
                print(f"{RED}Failed to upload Key: Permission Denied - Requires publickey{RESET}")
            elif result.returncode != 0:
                if "Permission denied, please try again." in result_string:
                    print(f"{RED}Failed to upload Key: Incorrect password inputted for sshpass{RESET}")
                else: 
                    print(f"{RED}Failed to upload key: Unknown Error{RESET}")
                    print("\n\n" + result_string + "\n\n")
            else:
                print(f"{GREEN}Key uploaded to {BLUE}{host_short}{RESET}")
        print("-----------------------------------------")
    cleanup_temp_files()

def cleanup_temp_files():
    print(f"{YELLOW}Cleaning up{RESET}")
    directories = [os.curdir, os.path.expanduser("~") + "/.ssh"]
    for directory in directories:
        for temp_file in os.listdir(directory):
            if temp_file.startswith('ssh-copy-id.'):
                try:
                    shutil.rmtree(os.path.join(directory, temp_file))
                except Exception as e:
                    print(f"{RED}Failed to clean up {temp_file}: {str(e)}{RESET}")

def get_sshpass_password():
    print(f"{YELLOW}Please input common password for use with sshpass{RESET}")
    try:
        return getpass()
    except KeyboardInterrupt:
        print(f"{ORANGE}\nExiting...{RESET}")
        exit()

def get_hosts(config_path, allow_proxies = False):
    hosts = []
    all_hosts_files = get_configs(config_path)

    if not allow_proxies:
        all_hosts_files[:] = [file for file in all_hosts_files if "proxy" not in str(file)]

    for host_file in all_hosts_files:
        if host_file.name == ".DS_Store":
            continue
        try:
            with open(host_file, 'r') as file:
                contents = file.read()
                hosts.extend(re.split(r"(?=[hH]ost )", contents))
        except Exception:
            print(f"{RED}Unable to parse file {host_file}")
            traceback.print_exc()
    return hosts

def get_configs(path: Path):
    config_paths = []
    for item in path.iterdir():
        if item.is_dir():
            config_paths.extend(get_configs(item))
        else:
            config_paths.append(item)
    return config_paths


def send_hosts(config_path):
    bastions = get_bastions(config_path)
    for bastion in bastions:
        print(f"{YELLOW}Sending config files to {BLUE}{bastion}{RESET}")
        scp_command = ["scp", "-r", config_path, f"{bastion}:.ssh"]
        subprocess_output = subprocess.run(scp_command, timeout=10, capture_output=True, text=True)
        if subprocess_output.returncode != 0:
            print(f"{RED}Error occured during transfer{RESET}")
        else:
            print(f"{GREEN}Files sent successfully{RESET}")
        print("-" * 40)

def get_bastions(config_path):
    all_hosts = get_hosts(config_path)
    bastions = []
    for host in all_hosts:
        host_names = [name for line in host.lower().splitlines() if "host " in line for name in line.split(" ")[1:]]
        if any("bastion" in host_name for host_name in host_names):
            bastions.append(host_names[0])
    return bastions

def create_proxies(path):
    proxy_folder = path / "proxy"
    ignore_list = [
        proxy_folder,
        path / f"{proxy_folder.name}-copy"
    ]

    all_hosts_files = get_configs(path)

    filtered_files = [
        f for f in all_hosts_files
        if f.name != ".DS_Store" and not any(f == ignore_path or f.is_relative_to(ignore_path) for ignore_path in ignore_list)
    ]

    set_proxy_folder(path, proxy_folder)

    try:
        make_proxy_files(path, proxy_folder, filtered_files)
    except Exception as e:
        revert_proxy(path, proxy_folder)
        print(f"{RED}Error has occured: Reverting back to previous version{RESET}")
        traceback.print_exc()
    finally:
        if (path / f"{proxy_folder.name}-copy").exists():
            shutil.rmtree(path / f"{proxy_folder.name}-copy")

def set_proxy_folder(path: Path, proxy_folder):
    proxy_copy_dir = path / f"{proxy_folder.name}-copy"

    if proxy_copy_dir.exists():
        shutil.rmtree(proxy_copy_dir)
    if proxy_folder.exists():
        shutil.move(proxy_folder, proxy_copy_dir)
    os.mkdir(proxy_folder)

def modify_host_line(match, bastion):
    host_keyword = match.group(1)
    words = match.group(2).split()
    modified_words = " ".join(f"{bastion}-{word}" for word in words)
    proxy_jump_line = f"    proxyJump {bastion}"    
    return f"{host_keyword} {modified_words}\n{proxy_jump_line}"

def make_proxy_files(config_path, proxy_folder, filtered_files):
    bastions = get_bastions(config_path)
    for bastion in bastions:
        print(f"{YELLOW}Creating proxy SSH Hosts for {BLUE}{bastion}{RESET}")
        bastion_proxy_path = proxy_folder / bastion
        os.mkdir(bastion_proxy_path)
        for host_file in filtered_files:
            file = open(host_file, "r")
            file_contents = file.read()

            new_file_contents = re.sub(
                r"(?i)^(Host) ([^\n]+)",
                lambda match: modify_host_line(match, bastion),
                file_contents,
                flags=re.MULTILINE
            )
            proxy_file = open(bastion_proxy_path / host_file.name, "x")
            proxy_file.write(new_file_contents)
            proxy_file.close()
        print(f"{GREEN}Proxy Hosts created for {BLUE}{bastion}{RESET}")
        print("-" * 40)
    create_config_file(bastions, proxy_folder)

def create_config_file(bastions, proxy_folder):
    file = open(os.path.expanduser("~/.ssh/config"), "w")
    relative_config_dir = "config.d/"
    file_contents = f"include {relative_config_dir}* "
    relative_proxy_dir = f"{relative_config_dir}{proxy_folder}"
    for bastion in bastions:
        relative_bastion_dir = f"{relative_proxy_dir}/{bastion}/* "
        file_contents += relative_bastion_dir
    file.write(file_contents)
    file.close()


def revert_proxy(path, proxy_folder):
    proxy_copy_dir = path / f"{proxy_folder.name}-copy"
    if os.path.exists(proxy_copy_dir):
        if os.path.exists(proxy_folder):
            shutil.rmtree(proxy_folder)
        shutil.move(proxy_copy_dir, proxy_folder)


if __name__ == "__main__":
    parser = ArgumentParser(description="SSH Config helper & send proxy keys")
    parser.add_argument("-p", "--proxy", action="store_true", help="Create proxy based off of configs")
    parser.add_argument("-b", "--bastion", action="store_true", help="Send config to bastions")
    parser.add_argument("-k", "--key", type=str, help="Add key to host authorised keys")
    parser.add_argument("-s", "--sshpass", action="store_true", help="Use sshpass for sending key")
    args = parser.parse_args()
    main(args)