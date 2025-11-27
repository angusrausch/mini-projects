#include <iostream>
#include "cli_request.h"
#include <string_view>
#include <atomic>
#include <csignal>

std::atomic<bool> stop_flag(false);

void signal_handler(int) {
    stop_flag = true;
}

int main(int argc, char *argv[]) {
    std::signal(SIGINT, signal_handler);

    if (argc > 1 && std::string_view(argv[1]) == "--cli") {
            CliRequest cli(argc, argv);
            return cli.run();
    } else {
        std::cout << "Currently no support for non-cli requests.\nPlease use --cli";
    }
    return 0;
}