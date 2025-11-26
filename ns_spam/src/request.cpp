#include "request.h"
#include <arpa/inet.h>
#include <chrono>
#include <cstring>
#include <iostream>
#include <sys/socket.h>
#include <unistd.h>
#include <thread>

void Request::encode_domain(const std::string &domain, std::vector<uint8_t> &buffer) {
    size_t start = 0, end;
    while ((end = domain.find('.', start)) != std::string::npos) {
        buffer.push_back(end - start);
        for (size_t i = start; i < end; ++i) buffer.push_back(domain[i]);
        start = end + 1;
    }
    buffer.push_back(domain.size() - start);
    for (size_t i = start; i < domain.size(); ++i) buffer.push_back(domain[i]);
    buffer.push_back(0);
}

int Request::make_request(const std::string &domain, const bool valid_site) {
        int sock = socket(AF_INET, SOCK_DGRAM, 0);
    if (sock < 0) {
        if (verbose) {
            output("Failed to create socket\n");
        }
        return -1;
    }

    sockaddr_in addr{};
    addr.sin_family = AF_INET;
    addr.sin_port = htons(53);
    if (inet_pton(AF_INET, nameserver.c_str(), &addr.sin_addr) <= 0) {
        if (verbose) {
            output("Invalid nameserver IP\n");
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
            output("Failed to send DNS query\n");
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
            output("DNS query timed out\n");
        }
        close(sock);
        return -1;
    }

    uint8_t buffer[512];
    ssize_t n = recvfrom(sock, buffer, sizeof(buffer), 0, nullptr, nullptr);
    if (n <= 0) {
        if (verbose) {
            output("Failed to receive DNS response\n");
        }
        close(sock);
        return -1;
    }

    if (n >= sizeof(DNSHeader)) {
        DNSHeader* resp_header = reinterpret_cast<DNSHeader*>(buffer);
        uint16_t ancount = ntohs(resp_header->ancount);
        if (ancount <= 0 && valid_site) {
            if (verbose) {
                output("Site NOT found in DNS response\n");
            }
            return -1;
        }
    }

    auto end = std::chrono::high_resolution_clock::now();
    close(sock);

    return std::chrono::duration_cast<std::chrono::milliseconds>(end - start).count();

}

bool Request::test_valid() {
    return make_request(domain, true) != -1;
}

void Request::worker(int requests) {
    for (int i = 0; i < requests; i++){
        make_request(domain, false);
    }
}

int Request::start_requests() {
    int requests_per_thread = requests / threads;
    int requests_remainder = requests % threads;

    std::vector<std::thread> thread_pool;

    for (int i = 0; i < threads; i++) {
        int worker_threads = requests_per_thread;
        if (i < requests_remainder) {
            worker_threads++;
        }

        thread_pool.emplace_back([this, worker_threads]() {
            worker(worker_threads);
        });
    }

    for (auto &t : thread_pool) {
        t.join();
    }

    return 0;
}

void Request::output(std::string str) {}