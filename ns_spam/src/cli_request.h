#pragma once

#include "request.h"
#include <string>

class CliRequest : public Request {
public:
    CliRequest(int argc, char *argv[]);
    int run();

private:
    void print_welcome();
    void output(std::string str) override;
};
