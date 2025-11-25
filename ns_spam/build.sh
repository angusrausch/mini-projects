#!/bin/bash

DIR="$(dirname "$(realpath "$0")")"

mkdir -p "$DIR/target"

clang++ -std=c++23 "$DIR/src/"*.cpp -o "$DIR/target/ns-spam" 