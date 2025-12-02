#!/bin/bash

DIR="$(dirname "$(realpath "$0")")"

mkdir -p "$DIR/target"
cd "$DIR"

cmake -S . -B target
cmake --build target