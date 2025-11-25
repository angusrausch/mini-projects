#include <arpa/inet.h>
#include <chrono>
#include <cstring>
#include <iostream>
#include <string>
#include <sys/socket.h>
#include <unistd.h>

#include "lib.h"

// Minimal DNS header for A record query
struct DNSHeader {
    uint16_t id;
    uint16_t flags;
    uint16_t qdcount;
    uint16_t ancount;
    uint16_t nscount;
    uint16_t arcount;
};

void encode_domain(const std::string &domain, std::vector<uint8_t> &buffer) {
    size_t start = 0, end;
    while ((end = domain.find('.', start)) != std::string::npos) {
        buffer.push_back(end - start);
        for (size_t i = start; i < end; ++i) buffer.push_back(domain[i]);
        start = end + 1;
    }
    buffer.push_back(domain.size() - start);
    for (size_t i = start; i < domain.size(); ++i) buffer.push_back(domain[i]);
    buffer.push_back(0); // null terminator
}

int make_request(const std::string &nameserver, const std::string &domain, const int timeout, const bool verbose, const bool valid_site) {
    int sock = socket(AF_INET, SOCK_DGRAM, 0);
    if (sock < 0) {
        if (verbose) {
            std::cerr << "Failed to create socket\n";
        }
        return -1;
    }

    sockaddr_in addr{};
    addr.sin_family = AF_INET;
    addr.sin_port = htons(53);
    if (inet_pton(AF_INET, nameserver.c_str(), &addr.sin_addr) <= 0) {
        if (verbose) {
            std::cerr << "Invalid nameserver IP\n";
        }
        close(sock);
        return -1;
    }

    // Build DNS query
    DNSHeader header{};
    header.id = htons(0x1234);
    header.flags = htons(0x0100); // standard query
    header.qdcount = htons(1);

    std::vector<uint8_t> packet(sizeof(DNSHeader));
    std::memcpy(packet.data(), &header, sizeof(DNSHeader));
    encode_domain(domain, packet);
    // Type A (1), Class IN (1)
    packet.push_back(0); packet.push_back(1); // Type A
    packet.push_back(0); packet.push_back(1); // Class IN

    auto start = std::chrono::high_resolution_clock::now();

    if (sendto(sock, packet.data(), packet.size(), 0,
               reinterpret_cast<sockaddr*>(&addr), sizeof(addr)) < 0) {
        if (verbose) {
            std::cerr << "Failed to send DNS query\n";
        }
        close(sock);
        return -1;
    }

    // Wait for response with 2-second timeout
    fd_set read_fds;
    FD_ZERO(&read_fds);
    FD_SET(sock, &read_fds);
    timeval tv{};
    tv.tv_sec = timeout;
    tv.tv_usec = 0;

    int ready = select(sock + 1, &read_fds, nullptr, nullptr, &tv);
    if (ready <= 0) {
        if (verbose) {
            std::cerr << "DNS query timed out\n";
        }
        close(sock);
        return -1;
    }

    uint8_t buffer[512];
    ssize_t n = recvfrom(sock, buffer, sizeof(buffer), 0, nullptr, nullptr);
    if (n <= 0) {
        if (verbose) {
            std::cerr << "Failed to receive DNS response\n";
        }
        close(sock);
        return -1;
    }

    auto end = std::chrono::high_resolution_clock::now();
    close(sock);

    return std::chrono::duration_cast<std::chrono::milliseconds>(end - start).count();
}

bool test_valid(Config config) {
    int time_taken = make_request(config.nameserver, config.domain, 5, true, true);

    return time_taken != -1;
}

int run(Config config) {
    int time_ms = make_request(config.nameserver, config.domain, 5, config.verbose);
    std::cout << "DNS query took " << time_ms << " ms\n";
    return 0;
}
 