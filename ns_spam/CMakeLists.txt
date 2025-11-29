cmake_minimum_required(VERSION 3.16)
project(ns_spam LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 23)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

file(GLOB SRC_FILES "src/*.cpp")
add_executable(ns-spam ${SRC_FILES})
