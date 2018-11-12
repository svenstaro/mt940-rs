#!/usr/bin/env bash

# Usage:
# update_test_output.sh <input_dir_with_sta_files>

for file in "$1"/*.sta; do
    cargo run --bin sta2json -- "$file" "${file%.sta}.json"
done
