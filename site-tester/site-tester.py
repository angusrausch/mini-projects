import argparse
import asyncio
import httpx
import json
import os
import random
import re
import time
from urllib.parse import urljoin, urlparse
from statistics import mean
from enum import Enum
import string
import sys

class RequestType(Enum):
    GET = "get"
    POST = "post"

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
ORANGE  = "\033[38;5;202m"

class AsyncApp:
    def __init__(self, args):
        self.url, self.url_normalised = self._normalize_url(args.url)
        self.total_requests = args.number
        self.follow_links = args.follow_links
        self.concurrency = args.processes
        self.payload = AsyncApp.check_payload(args.payload)
        self.filters = ["http", "static/", "cdn/", "googleapis", "."]
        self.link_regex = re.compile(r'href="([^"]*)"')
        self.verify_ssl = args.ignore_ssl
        self.timeout = args.timeout

        self.reply_options()

        self.times = []
        self.lock = asyncio.Lock()
        self.sites_visited = []
        self.request_type = RequestType(args.type.lower())
        self.request_func = self._get_request_func()

    def reply_options(self):
        ascii_banner = fr"""{MAGENTA}{BOLD}

       _____ _____ _______ ______   _______ ______  _____ _______ ______ _____  
      / ____|_   _|__   __|  ____| |__   __|  ____|/ ____|__   __|  ____|  __ \ 
     | (___   | |    | |  | |__ ______| |  | |__  | (___    | |  | |__  | |__) |
      \___ \  | |    | |  |  __|______| |  |  __|  \___ \   | |  |  __| |  _  / 
      ____) |_| |_   | |  | |____     | |  | |____ ____) |  | |  | |____| | \ \ 
     |_____/|_____|  |_|  |______|    |_|  |______|_____/   |_|  |______|_|  \_\

            {RESET}
            """

        # Build message
        string = f"{YELLOW}Thank you for using {MAGENTA}Site-Tester{RESET}.\n"
        string += f"{YELLOW}This application should only be run on websites you have permission from the owner to use.{RESET}\n"
        if self.url_normalised:
            string += f"{YELLOW}No protocol given. Defaulting to {BLUE}\"https\"{RESET}\n"
        string += f"{YELLOW}You have selected website {BOLD}{BLUE}{self.url}{RESET} {YELLOW}to run on.{RESET}\n"
        string += f"{YELLOW}Continuing will make {BOLD}{BLUE}{self.total_requests}{RESET} {YELLOW}requests to the server using {BOLD}{BLUE}{self.concurrency}{RESET} {YELLOW}threads{RESET}\n"
        if self.follow_links:
            string += f"{YELLOW}You have also selected to follow a random link on the page{RESET}\n"
        if self.timeout != 10:
            string += f"{YELLOW}Request timeout has been set at {self.timeout}.{RESET}\n"
        if not self.verify_ssl:
            string += f"{YELLOW}Ignoring SSL errors{RESET}\n"

        # Print splash
        print(ascii_banner)
        print(f"{MAGENTA}{'-'*114}{RESET}")
        print(string)
        print(f"{MAGENTA}{'-'*114}{RESET}")
        if sys.stdin.isatty(): 
            print(f"\n{YELLOW}Press enter to begin or CTRL+C to exit{RESET}")
            try:
                input()
            except KeyboardInterrupt:
                print(f"\n{GREEN}Exiting gracefully{RESET}")
                exit()
        else:
            print(f"\n{ORANGE}Non-interactive mode detected{RESET}\n")

    def _get_request_func(self):
        if self.request_type == RequestType.GET:
            return lambda client, url: client.get(url)
        elif self.request_type == RequestType.POST:
            return lambda client, url: client.post(url, data={})

    @staticmethod
    def check_payload(payload):
        if not payload:
            return None
        try:
            return json.loads(payload)
        except json.JSONDecodeError:
            if os.path.isfile(payload):
                with open(payload, "r") as file:
                    contents = file.read()
                    return AsyncApp.check_payload(contents)
            print("Invalid Payload, please try again")
            exit()

    @staticmethod
    def _normalize_url(url):
        if not url.startswith(("http://", "https://")):
            return f"https://{url}", True
        return url, False

    def check_url(self):
        print(f"{YELLOW}Probing: {BLUE}{self.url}{RESET}")
        try:
            start = time.perf_counter()
            response = httpx.get(self.url, timeout=self.timeout, verify=self.verify_ssl)
            duration = time.perf_counter() - start
        except Exception as e:
            print(f"{RED}Error probing URL: {e}{RESET}")
            return False, 0
        else:
            print(f"{GREEN}Connected to {BLUE}{self.url} {GREEN}(initial check in {BLUE}{duration:.2f}{GREEN}s){RESET}")
            return response.status_code == 200

    async def run(self):
        ok = self.check_url()
        if not ok:
            print("Initial URL check failed.")
            return

        # Calculate how many requests each worker should perform
        requests_per_worker = self.total_requests // self.concurrency
        remainder = self.total_requests % self.concurrency

        tasks = []
        for i in range(self.concurrency):
            count = requests_per_worker + (1 if i < remainder else 0)
            tasks.append(self.worker(count))
        await asyncio.gather(*tasks)

        self.report()

    async def worker(self, request_count):
        url = self.url
        async with httpx.AsyncClient(timeout=self.timeout, verify=self.verify_ssl) as client:
            for _ in range(request_count):
                url = await self.make_request(url, client)


    async def make_request(self, url, client):
        try:
            start = time.perf_counter()
            if url is None:
                url = self.url
            response = await self.request_func(client, url)
            duration = time.perf_counter() - start

            if response.status_code != 200 and self.request_type == "get":
                print(f"{ORANGE}[WARN] {url} returned {response.status_code} in {duration:.2f}s | adding to blacklist{RESET}")
                self.filters.append(urlparse(url).path)
                return self.url

            async with self.lock:
                self.times.append(duration)

            if self.follow_links:
                return self.pick_next_url(response.text)
            return self.url

        except Exception as e:
            print(f"{RED}[ERROR] Failed request to {url}: {e}{RESET}")
            return self.url

    def pick_next_url(self, html):
        links = self.link_regex.findall(html)
        valid_links = [link for link in links if not any(f in link for f in self.filters)]
        if not valid_links:
            return self.url
        new_url = urljoin(self.url, random.choice(valid_links))
        if new_url not in self.sites_visited:
            self.sites_visited.append(new_url)
        return 

    def report(self):
        if not self.times:
            print(f"{RED}No successful requests recorded.{RESET}")
            return

        avg = mean(self.times)
        max_time = max(self.times)

        print(f"\n{GREEN}Completed {BLUE}{len(self.times)}{GREEN} requests.{RESET}")
        print(f"{GREEN}Average response time: {BLUE}{avg:.3f}{GREEN}s{RESET}")
        print(f"{GREEN}Maximum response time: {BLUE}{max_time:.3f}{GREEN}s{RESET}")
        if self.follow_links:
            print(f"{GREEN}This included {BLUE}{len(self.sites_visited)} {GREEN}unique pages{RESET}")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Async Website Performance Tester")
    parser.add_argument("--url", type=str, required=True, help="Target URL to test")
    parser.add_argument("-f", "--follow-links", action="store_true", help="Follow hyperlinks on the page")
    parser.add_argument("-n", "--number", type=int, default=100, help="Total number of requests to make")
    parser.add_argument("-p", "--processes", type=int, default=10, help="Number of concurrent workers")
    parser.add_argument("--type", type=str, choices=["get", "post"], default="get", help="HTTP method to use: get or post")
    parser.add_argument("--payload", type=str, default="", help="Raw JSON or path to file with JSON")
    parser.add_argument("--ignore-ssl", action="store_false", help="Disable SSL check")
    parser.add_argument("--timeout", type=int, default=10, help="Timeout for each individual request before failing in seconds")
    args = parser.parse_args()
    app = AsyncApp(args)
    asyncio.run(app.run())
