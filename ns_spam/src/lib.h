#pragma once

#include <string>

struct Config {
    std::string nameserver;
    std::string domain = "google.com";
    int requests = 100;
    int threads = 10;
    bool verbose = false;
    bool random = false;
};

int run(Config config);

bool test_valid(Config config);