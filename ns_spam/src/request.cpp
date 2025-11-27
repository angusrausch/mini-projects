#include "request.h"
#include <arpa/inet.h>
#include <chrono>
#include <cstring>
#include <iostream>
#include <sys/socket.h>
#include <unistd.h>
#include <thread>
#include <algorithm>
#include <tuple>
#include <cmath>
#include <atomic>

extern std::atomic<bool> stop_flag;

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

std::tuple<int, int, int, int> Request::worker(int requests) {
    int failed = 0;
    int total_time = 0;
    int max_time = 0;
    int successful_requests = 0;
    for (int i = 0; (i < requests || endless) && !stop_flag; i++){
        int time_taken = make_request(domain, false);
        if (time_taken == -1) {
            failed++;
        } else {
            successful_requests ++;
            total_time += time_taken;
            max_time = std::max(max_time, time_taken);
        }
    }
    return std::tuple(total_time, failed, max_time, successful_requests);
}


int Request::start_requests() {
    int requests_per_thread = requests / threads;
    int requests_remainder = requests % threads;

    std::vector<std::thread> thread_pool;
    std::vector<std::tuple<int, int, int, int>> results(threads);

    for (int i = 0; i < threads; i++) {
        int worker_threads = requests_per_thread;
        if (i < requests_remainder) {
            worker_threads++;
        }

        thread_pool.emplace_back([this, worker_threads, &results, i]() {
            results[i] = worker(worker_threads);
        });
    }

    long long total_time = 0;
    int failed = 0, max_time = 0, successful_requests = 0;

    for (int i = 0; i < threads; i++) {
        thread_pool[i].join();
        auto [total_time_thread, failed_thread, max_time_thread, successful_requests_thread] = results[i];
        total_time += total_time_thread;
        failed += failed_thread;
        max_time = std::max(max_time, max_time_thread);
        successful_requests += successful_requests_thread;
    }

    // Create summary
    double average = std::round(
        ((double)total_time / successful_requests) / 1000.0 * 100.0
    ) / 100.0;
    double max_time_seconds = std::round(
        (double) max_time / 1000.0 * 100.0
    ) / 100.0;
    output(std::format("\nTotal of {} requests returned.\n"
        "{} requests failed\n"
        "An average time per successful request of {} seconds\n"
        "Longest successful request took {} seconds\n", 
        successful_requests, failed, average, max_time_seconds));

    return 0;
}


void Request::output(std::string str) {}