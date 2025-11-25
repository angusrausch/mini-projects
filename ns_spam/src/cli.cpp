#include "cli.h"
#include "lib.h"
#include <iostream>
#include <string>
#include <format>

// Declare CLI colours
std::string RED     = "\033[31m";
std::string GREEN   = "\033[32m";
std::string YELLOW  = "\033[33m";
std::string BLUE    = "\033[34m";
std::string MAGENTA = "\033[35m";
std::string CYAN    = "\033[36m";
std::string RESET   = "\033[0m";
std::string BOLD    = "\033[1m";
std::string ORANGE = "\033[38;5;208m";

Config get_cli_args(int argc, char *argv[]) {
    Config config;

    for (int i = 1; i < argc; ++i) {
        std::string arg = argv[i];

        if (arg == "--nameserver" && i + 1 < argc) {
            config.nameserver = argv[++i];
        } else if (arg == "--domain" && i + 1 < argc) {
            config.domain = argv[++i];
        } else if (arg == "--requests" && i + 1 < argc) {
            config.requests = std::stoi(argv[++i]);
        } else if (arg == "--threads" && i + 1 < argc) {
            config.threads = std::stoi(argv[++i]);
        } else if (arg == "--verbose" || arg == "-v") {
            config.verbose = true;
        } else if (arg == "--random" || arg == "-r") {
            config.random = true;
        } else {
            if (arg != "--cli") {
                std::cerr << "Unknown argument: " << arg << "\n";
            }
        }
    }

    // Nameserver parameter has no default
    if (config.nameserver == "") {
        std::cout << std::format("{}ERROR: Nameserver argument required use --nameserver to specify nameserver.{}", RED, RESET);
        exit(1);
    }

    return config;
}

void print_welcome(Config config) {
    std::string ascii_banner = std::format("{}{}"
    " _   _  _____       _____ _____        __  __ \n"
    "| \\ | |/ ____|     / ____|  __ \\ /\\   |  \\/  |\n"
    "|  \\| | (___ _____| (___ | |__) /  \\  | \\  / |\n"
    "| . ` |\\___ \\______\\___ \\|  ___/ /\\ \\ | |\\/| |\n"
    "| |\\  |____) |     ____) | |  / ____ \\| |  | |\n"
    "|_| \\_|_____/     |_____/|_| /_/    \\_\\_|  |_|\n\n\n",
    BOLD, CYAN);
    std::cout << ascii_banner;

    std::string str = std::format(
        "{}Thank you for using {}NS-Spam{}.\n",
        YELLOW, MAGENTA, RESET
    );

    str += std::format(
        "{}This application should only be run on nameservers you have permission from the owner to use.{}\n",
        YELLOW, RESET
    );

    str += std::format(
        "{}You have selected nameserver {}{}{}{}{} to run on.{}\n",
        YELLOW, BOLD, BLUE, config.nameserver, RESET, YELLOW, RESET
    );

    if (config.random) {
        str += std::format(
            "{}Random option selected. {}{}This will create random subdomains off of your domain {}{}{}{}{} with a length of 3-7 characters.{}\n",
            YELLOW, RESET, YELLOW, BOLD, BLUE, config.domain, RESET, YELLOW, RESET
        );
    } else {
        str += std::format(
            "{}You have selected the domain {}{}{}{}{} to run on.{}\n",
            YELLOW, BOLD, BLUE, config.domain, RESET, YELLOW, RESET
        );
    }
    str += std::format(
        "{}You have selected to make {}{}{}{}{} requests using {}{}{}{}{} threads.{}\n",
        YELLOW, BOLD, BLUE, config.requests, RESET, YELLOW, BOLD, BLUE, config.threads, RESET, YELLOW, RESET
    );

    str += "\n\n";
    std::cout << str;
}

int cli(int argc, char *argv[]) {
    Config config = get_cli_args(argc, argv);

    print_welcome(config);
    
    std::cout << std::format("{}Testing nameserver and domain are valid.\n{}", YELLOW, ORANGE);
    if (test_valid(config)) {
        std::cout << std::format("\n{}Request successful. Starting requests now{}\n", GREEN, RESET);
    } else {
        std::cout << std::format("\n{}Request failed. Check nameserver and domain are valid{}", RED, RESET);
        exit(1);
    }

    run(config);

    return 0;
}