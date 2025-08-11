import random
import argparse
import string
import subprocess
from threading import Thread, active_count
from time import sleep, time_ns

# Declare globals
alphabet = string.ascii_lowercase
RED     = "\033[31m"
GREEN   = "\033[32m"
YELLOW  = "\033[33m"
BLUE    = "\033[34m"
MAGENTA = "\033[35m"
CYAN    = "\033[36m"
RESET   = "\033[0m"
BOLD    = "\033[1m"

class App:

    def __init__(self, args):
        try:
            self.nameserver = args.nameserver
            self.random = args.random
            self.number = args.number
            self.domain = args.domain
            self.verbose = args.verbose
            if args.threads:
                self.max_threads = int(args.threads)
            else:
                self.max_threads = False
        except TypeError:
            print("Invalid data type for input")

        self.reply_options()

        self.run()

    def reply_options(self):
        ascii_banner = fr"""{CYAN}{BOLD}
             _   _  _____       _____ _____        __  __ 
            | \ | |/ ____|     / ____|  __ \ /\   |  \/  |
            |  \| | (___ _____| (___ | |__) /  \  | \  / |
            | . ` |\___ \______\___ \|  ___/ /\ \ | |\/| |
            | |\  |____) |     ____) | |  / ____ \| |  | |
            |_| \_|_____/     |_____/|_| /_/    \_\_|  |_|                                   
            {RESET}
            """

        # Build message
        string = f"{YELLOW}Thank you for using {MAGENTA}NS-Spam{RESET}.\n"
        string += f"{YELLOW}This application should only be run on nameservers you have permission from the owner to use.{RESET}\n"
        string += f"{YELLOW}You have selected nameserver {BOLD}{BLUE}{self.nameserver}{RESET} {YELLOW}to run on.{RESET}\n"

        if self.random:
            string += f"{YELLOW}Random option selected.{RESET} {YELLOW}This will create random subdomains off of your domain {BOLD}{BLUE}{self.domain}{RESET} {YELLOW} with a length of 3-7 characters.{RESET}\n"
        else:
            string += f"{YELLOW}You have selected the domain {BOLD}{BLUE}{self.domain}{RESET} {YELLOW}to run on.{RESET}\n"

        # Print splash
        print(ascii_banner)
        print(f"{MAGENTA}{'-'*114}{RESET}")
        print(string)
        print(f"{MAGENTA}{'-'*114}{RESET}")
        print(f"\n{YELLOW}Press enter to begin or CTRL+C to exit{RESET}")
        try:
            input()
        except KeyboardInterrupt:
            print(f"\n{GREEN}Exiting gracefully{RESET}")
            exit()

    def run(self):
        for i in range(self.number):
            if self.max_threads and active_count() >= self.max_threads - 1:
                sleep(0.1)
            print(f"{CYAN}Starting thread {i}{RESET}")
            thread = Thread(target=self.process, args=(i,))
            thread.start()

    def process(self, index):
        check_domain = self.domain
        if self.random:
            check_domain = f"{self.randomise()}.{check_domain}"
        if self.verbose:
            print(f"{YELLOW}Performing lookup on {check_domain}{RESET}")
        ip = self.nslookup(check_domain)
        if self.verbose:
            print(f"{GREEN}Thread {index} has finished with result {BOLD}{ip} {GREEN}for domain {BOLD}{BLUE}{check_domain}{RESET}")
        else:
            print(f"{GREEN}Thread {index} finished{RESET}")

    def nslookup(self, domain):
        try:
            start_time = time_ns()
            nslookup_output = subprocess.check_output(
                ["nslookup", domain, self.nameserver], stderr=subprocess.STDOUT  # nosec B607 B603
            ).decode("utf-8")
            end_time = time_ns()
            time_taken = round((end_time - start_time) / 1000000000, 2)
        except subprocess.CalledProcessError:
            end_time = time_ns()
            time_taken = round((end_time - start_time) / 1000000000, 2)
            return f"{RED}No Return | {time_taken}s to complete{RESET}"
        else:
            if self.verbose:
                if self.verbose:    
                    ip = None
                    for line in nslookup_output.splitlines():
                        if "Address:" in line and str(self.nameserver) not in line:
                            return f"{BLUE}{line.split("Address:")[1].strip()} | {time_taken}s to complete{RESET}"
        return

    def randomise(self):
        length = random.randint(3, 7)
        string = ""
        for i in range(length):
            string += alphabet[random.randint(0, 25)]
        return string

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=f"{MAGENTA}Asyncronous DNS Namerserver Tester{RESET}")
    parser.add_argument("--nameserver", type=str, required=True, help="Target nameserver")
    parser.add_argument("-d", "--domain", type=str, default="google.com", help="Domain to ping. (Random will create sub domains off of this)")
    parser.add_argument("-i", "--number", type=int, default=100, help="Total number of requests to make")
    parser.add_argument("-r", "--random", action="store_true", help="Create random sub domains to check")
    parser.add_argument("-v", "--verbose", action="store_true", help="More verbose output")
    parser.add_argument("-t", "--threads",)
    args = parser.parse_args()
    app = App(args)
