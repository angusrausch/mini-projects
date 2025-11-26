#include "cli_request.h"
#include <iostream>
#include <format>

// CLI colours
static const std::string RED     = "\033[31m";
static const std::string GREEN   = "\033[32m";
static const std::string YELLOW  = "\033[33m";
static const std::string BLUE    = "\033[34m";
static const std::string MAGENTA = "\033[35m";
static const std::string CYAN    = "\033[36m";
static const std::string RESET   = "\033[0m";
static const std::string BOLD    = "\033[1m";
static const std::string ORANGE  = "\033[38;5;208m";

CliRequest::CliRequest(int argc, char *argv[]) {
    for (int i = 1; i < argc; ++i) {
        std::string arg = argv[i];

        if (arg == "--nameserver" && i + 1 < argc) {
            nameserver = argv[++i];
        } else if (arg == "--domain" && i + 1 < argc) {
            domain = argv[++i];
        } else if ((arg == "--requests" || arg == "-n") && i + 1 < argc) {
            requests = std::stoi(argv[++i]);
        } else if ((arg == "--threads" || arg == "-t") && i + 1 < argc) {
            threads = std::stoi(argv[++i]);
        } else if (arg == "--verbose" || arg == "-v") {
            verbose = true;
        } else if (arg == "--random" || arg == "-r") {
            random = true;
        } else if (arg == "--timeout" && i + 1 < argc) {
            timeout = std::stoi(argv[++i]);
        } else if (arg != "--cli") {
            std::cerr << std::format("{}Unknown argument: {}{}\n", RED, arg, RESET);
            exit(1);
        }
    }

    if (nameserver.empty()) {
        std::cerr << std::format(
            "{}ERROR: --nameserver is required{}\n",
            RED, RESET
        );
        exit(1);
    }
}

void CliRequest::print_welcome() {
    std::string ascii_banner = std::format(
        "{}{}"
        " _   _  _____       _____ _____        __  __ \n"
        "| \\ | |/ ____|     / ____|  __ \\ /\\   |  \\/  |\n"
        "|  \\| | (___ _____| (___ | |__) /  \\  | \\  / |\n"
        "| . ` |\\___ \\______\\___ \\|  ___/ /\\ \\ | |\\/| |\n"
        "| |\\  |____) |     ____) | |  / ____ \\| |  | |\n"
        "|_| \\_|_____/     |_____/|_| /_/    \\_\\_|  |_|\n\n\n{}",
        BOLD, CYAN, RESET
    );
    std::cout << ascii_banner;

    std::cout << std::format(
        "{}Thank you for using {}NS-Spam{}.\n",
        YELLOW, MAGENTA, RESET
    );

    std::cout << std::format(
        "{}This application should only be run on nameservers you have permission "
        "from the owner to use.{}\n",
        YELLOW, RESET
    );

    std::cout << std::format(
        "{}You have selected nameserver {}{}{}{}{}{}\n",
        YELLOW, BOLD, BLUE, nameserver, RESET, YELLOW, RESET
    );

    if (random) {
        std::cout << std::format(
            "{}Random mode enabled — generating random subdomains under {}{}{}{}\n",
            YELLOW, BLUE, domain, RESET, YELLOW
        );
    } else {
        std::cout << std::format(
            "{}Domain selected: {}{}{}{}\n",
            YELLOW, BLUE, domain, RESET, YELLOW
        );
    }

    std::cout << std::format(
        "{}Requests: {}{}{} | Threads: {}{}{}\n\n",
        YELLOW, BLUE, requests, YELLOW, BLUE, threads, RESET
    );
}

int CliRequest::run() {
    print_welcome();

    std::cout << std::format("{}Testing nameserver and domain...\n{}", YELLOW, ORANGE);

    if (test_valid()) {
        output(std::format("{}Request Successful{}", GREEN, RESET));
    } else {
        std::cerr << std::format(
            "{}Request failed — invalid nameserver or domain{}\n",
            RED, RESET
        );
        return 1;
    }

    std::cout << std::format(
        "\n\n{}Starting workload...{}\n",
        YELLOW, RESET
    );

    start_requests();

    std::cout << std::format(
        "{}Requests completed\n{}",
        GREEN, RESET
    );

    return 0;
}

void CliRequest::output(std::string str) {
    std::cout << str;
}