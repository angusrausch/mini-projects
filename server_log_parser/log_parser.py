import argparse
import gzip
import os
import re
from datetime import datetime, timedelta

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
        if args.file:
            self.log_string = self.read_log(args.file)
        elif args.directory:
            self.log_string = self.combine_logs(args.directory)
        else:
            print(f"{RED}No log files selected.\nExiting Now.{RESET}")
            exit()
        if  not self.log_string:
            print(f"{RED}No Log files inputted\nExiting now!{RESET}")
        if args.list_players:
            self.list_players()
            exit()
            exit
        if args.player_time:
            self.check_valid_time_slot(args.time_slot)
            self.itterate_logs(args.player_time)
            self.find_play_time()
            self.print_time_played(args.player_time)

    def check_valid_time_slot(self, time_slot):
        if not time_slot:
            self.start_time_slot = None
            return
        try:
            self.start_time_slot = int(time_slot[0])
            self.end_time_slot = int(time_slot[1])
            if min(time_slot) < 0 or max(time_slot) > 23:
                raise ValueError()
        except ValueError:
            print("Time slot must be hours only and 24 hour time")
            exit()

    def list_players(self):
        player_joins = re.findall(r"UUID of player (\w+)", self.log_string)
        unique_players = list(set(player_joins))
        print(f"{GREEN}Listing all players played within logs{RESET}")
        for player in unique_players:
            print(f"{BLUE}{player}{RESET}")
        exit()

    def print_time_played(self, player):
        days = int(self.time_played / 86400)
        hours = int(self.time_played % 86400 / 3600)
        minutes = int(self.time_played % 3600 / 60)
        seconds = int(self.time_played % 60)
        parts = []

        if days:
            parts.append(f"{days} days")
            parts.append(f"{hours} hours")
            parts.append(f"{minutes} minutes")
            parts.append(f"{seconds} seconds")
        elif hours:
            parts.append(f"{hours} hours")
            parts.append(f"{minutes} minutes")
            parts.append(f"{seconds} seconds")
        elif minutes:
            parts.append(f"{minutes} minutes")
            parts.append(f"{seconds} seconds")
        else:
            parts.append(f"{seconds} seconds")

        duration_string = ", ".join(parts)
        print(f"{GREEN}The player {player} has played for a total of:{RESET}")
        print(BLUE + duration_string + RESET)
        if self.start_time_slot:
            print(f"Between the hours of {self.start_time_slot}-{self.end_time_slot} UTC Monday - Friday")
    
    def find_play_time(self):
        self.time_played = 0
        for session_start, session_end in zip(self.player_join_list, self.player_left_list):
            if self.start_time_slot == None:
                self.time_played += (session_end - session_start).total_seconds()
                continue

            if session_start.weekday() >= 5: # Do not do weekends
                continue

            slot_start = session_start.replace(hour=self.start_time_slot, minute=0, second=0, microsecond=0)
            if self.end_time_slot > self.start_time_slot:
                slot_end = session_start.replace(hour=self.end_time_slot, minute=0, second=0, microsecond=0)
            else:
                slot_end = (slot_start + timedelta(days=1)).replace(hour=self.end_time_slot)

            effective_start = max(session_start, slot_start)
            effective_end = min(session_end, slot_end)
            if effective_start < effective_end:
                self.time_played += (effective_end - effective_start).total_seconds()

    def itterate_logs(self, player):
        player_join_line_regex = rf"\[(\d{{2}}:\d{{2}}:\d{{2}})\] \[Server thread/INFO\]: {player} joined the game".lower()
        player_left_line_regex = rf"\[(\d{{2}}:\d{{2}}:\d{{2}})\] \[Server thread/INFO\]: {player} left the game".lower()
        self.player_join_list = []
        self.player_left_list = []
        split_logs = self.log_string.split("\n")
        log_date = datetime.now().date()
        for line in split_logs:
            if re.search(r"^\d{4}-\d{2}-\d{2}$", line[:10]) :
                log_date = datetime.strptime(line[:10], "%Y-%m-%d").date()
                continue
            join_match = re.search(player_join_line_regex, line.lower())
            if join_match:
                time_obj = datetime.strptime(join_match.group(1), "%H:%M:%S").time()
                self.player_join_list.append(datetime.combine(log_date, time_obj))
                continue
            left_match = re.search(player_left_line_regex, line.lower())
            if left_match:
                time_obj = datetime.strptime(left_match.group(1), "%H:%M:%S").time()
                self.player_left_list.append(datetime.combine(log_date, time_obj))
                continue

    def combine_logs(self, directory):
        log = ""
        latest_date = None
        date_pattern = re.compile(r"(\d{4}-\d{2}-\d{2})")  # matches YYYY-MM-DD

        try:
            for file in sorted(os.listdir(directory)):
                file_path = f"{directory}/{file}"

                match = date_pattern.search(file)
                if match:
                    file_date = datetime.strptime(match.group(1), "%Y-%m-%d").date()
                    if latest_date is None or file_date > latest_date:
                        latest_date = file_date

                if "latest" in file and latest_date:
                    next_day = latest_date + timedelta(days=1)
                    display_name = next_day.strftime("%Y-%m-%d")
                else:
                    display_name = file.split(".")[0]

                log += display_name + "\n"

                if ".gz" in file:
                    log += self.set_lines(self.read_compress_file(file_path))
                else:
                    log += self.read_file(file_path)
        except FileNotFoundError:
            print(f"{RED}Directory or file not found. Check directory and ensure contents are valid.\nExiting now{RESET}")
            exit()
        return log


    def read_log(self, file):
        try:
            if ".gz" in file:
                return self.set_lines(self.read_compress_file(file))
            else: 
                return self.read_file(file)
        except FileNotFoundError:
            print(f"{RED}File not found. Check file exists and is correct format.\nExiting now{RESET}")
            exit()

    def set_lines(self, contents):
        return contents.decode('utf-8')

    def read_compress_file(self, file):
        with gzip.open(file, 'rb') as f:
            return f.read()
        
    def read_file(self, file):
        with open(file, "r") as f:
            return f.read()

def is_date_filename(filename):
    try:
        datetime.strptime(filename.split(".")[0], "%Y-%m-%d-%d")
        return True
    except ValueError:
        return False

def parse_date_from_filename(filename):
    return datetime.strptime(filename.split(".")[0], "%Y-%m-%d-%d")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Crafty Log Parser")
    parser.add_argument("-f", "--file", type=str, help="Single Log File")
    parser.add_argument("-d", "--directory", type=str, help="Directory with log files")
    parser.add_argument("--player-time", type=str, help="Find time a player has been online")
    parser.add_argument("--list-players", action="store_true", help="List all players")
    parser.add_argument("--time-slot", type=int, nargs=2, metavar=("START_HOUR", "END_HOUR"), help="Time slot in UTC hours using 24 hour time")
    args = parser.parse_args()
    App(args)