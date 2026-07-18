#!/usr/bin/env bash
set -euo pipefail

# cargo run -- save && \

#DATA_PATH="./mlc-sample"
#DATA_PATH="./Sample Data"
DATA_PATH="./mlc-bwarm"

cargo build --release
BIN="./target/release/mlc-reader"
$BIN migrate -p "$DATA_PATH" && \

$BIN modify && \
$BIN enrich -m writer > /dev/null && \
$BIN enrich -m publisher > /dev/null && \
$BIN enrich -m share > /dev/null 


# cargo run -- update -p './updates.txt' && \
# turso db create mlc-dump --from-file './local.db'
