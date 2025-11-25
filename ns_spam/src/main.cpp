#include <iostream>
#include "cli.h"
#include <string_view>

int main(int argc, char *argv[]) {
    if (argc > 1 && std::string_view(argv[1]) == "--cli") {
        cli(argc, argv);
    } else {
        std::cout << "Currently no support for non-cli requests.\nPlease use --cli";
    }
    return 0;
}