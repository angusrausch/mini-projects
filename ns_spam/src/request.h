#pragma once

#include <string>
#include <vector>
#include <cstdint>

class Request {
public:
    std::string nameserver;
    std::string domain = "google.com";
    int requests = 100;
    int threads = 10;
    int timeout = 5;
    bool verbose = false;
    bool random = false;

    // Default constructor needed for CliRequest
    Request() = default;

    Request(const std::string& nameserver,
            const std::string& domain = "google.com",
            int requests = 100, int threads = 10,
            bool verbose = false, bool random = false)
        : nameserver(nameserver), domain(domain),
          requests(requests), threads(threads),
          verbose(verbose), random(random) {}

    bool test_valid();
    int start_requests();

protected:
    virtual void output(std::string str);

private:
    struct DNSHeader {
        uint16_t id;
        uint16_t flags;
        uint16_t qdcount;
        uint16_t ancount;
        uint16_t nscount;
        uint16_t arcount;
    };

    void encode_domain(const std::string &domain, std::vector<uint8_t> &buffer);
    int make_request(const std::string &domain, const bool valid_site);
    void worker(int requests);
};
