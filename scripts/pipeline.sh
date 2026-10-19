#!/usr/bin/env bash
set -euo pipefail

#TSV_STORAGE_PATH="./mlc-sample"
#TSV_STORAGE_PATH="./Sample Data"
TSV_STORAGE_PATH="./mlc-bwarm"

cargo build --release
BIN="./target/release/mlc-reader"

export SQLITE_TMPDIR="$PWD/sqlite-tmp"
export TMPDIR=$SQLITE_TMPDIR
export RUST_LOG=debug

mkdir -p "$SQLITE_TMPDIR"
chmod 700 "$SQLITE_TMPDIR"

nohup $BIN night-job --resume-step schema > job.log 2>&1 &
