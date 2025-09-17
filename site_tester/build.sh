#!/bin/bash

# Color codes
RED="\033[31m"
GREEN="\033[32m"
YELLOW="\033[33m"
BLUE="\033[34m"
MAGENTA="\033[35m"
CYAN="\033[36m"
RESET="\033[0m"
BOLD="\033[1m"
ORANGE="\033[38;5;202m"

echo -e "${MAGENTA}${BOLD}

       _____ _____ _______ ______   _______ ______  _____ _______ ______ _____  
      / ____|_   _|__   __|  ____| |__   __|  ____|/ ____|__   __|  ____|  __ \ 
     | (___   | |    | |  | |__ ______| |  | |__  | (___    | |  | |__  | |__) |
      \___ \  | |    | |  |  __|______| |  |  __|  \___ \   | |  |  __| |  _  / 
      ____) |_| |_   | |  | |____     | |  | |____ ____) |  | |  | |____| | \ \ 
     |_____/|_____|  |_|  |______|    |_|  |______|_____/   |_|  |______|_|  \_\

${RESET}"

echo -e "${CYAN}Building for local machine${RESET}"

spinner() {
    local pid=$1
    local delay=0.1
    local spinstr='|/-\'
    while kill -0 $pid 2>/dev/null; do
        local temp=${spinstr#?}
        printf " [%c]  " "$spinstr"
        spinstr=$temp${spinstr%"$temp"}
        sleep $delay
        printf "\b\b\b\b\b\b"
    done
}

echo -e "${YELLOW}    Building debug${RESET}"
cargo build --quiet 2>build_debug.err 1>/dev/null &
build_pid=$!
spinner $build_pid
wait $build_pid
debug_status=$?
if [ $debug_status -ne 0 ]; then
    echo -e "${RED}Debug build failed:${RESET}"
    cat build_debug.err
    exit 1
fi
rm -f build_debug.err

echo -e "${YELLOW}    Building release${RESET}"
cargo build --release --quiet 2>build_release.err 1>/dev/null &
build_pid=$!
spinner $build_pid
wait $build_pid
release_status=$?
if [ $release_status -ne 0 ]; then
    echo -e "${RED}Release build failed:${RESET}"
    cat build_release.err
    exit 1
fi
rm -f build_release.err

echo -e "${CYAN}Building for Linux x86_64${RESET}"
cross build --target x86_64-unknown-linux-gnu --release --quiet 2>build_linux.err 1>/dev/null &
build_pid=$!
spinner $build_pid
wait $build_pid
linux_status=$?
if [ $linux_status -ne 0 ]; then
    echo -e "${RED}Linux x86_64 build failed:${RESET}"
    cat build_linux.err
    exit 1
fi
rm -f build_linux.err

echo -e "${GREEN}Builds completed successfully.${RESET}"
