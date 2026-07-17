#!/usr/bin/env bash
set -euo pipefail

# cargo run -- save && \

DATA_PATH="./mlc-sample"
#DATA_PATH="./Sample Data"
#DATA_PATH="/mlc-bwarm"
cargo run -- migrate -p "$DATA_PATH" && \

cargo run -- modify && \
cargo run -- enrich -m writer && \
cargo run -- enrich -m publisher && \
cargo run -- enrich -m share 


# cargo run -- update -p './updates.txt' && \
# turso db create mlc-dump --from-file './local.db'
