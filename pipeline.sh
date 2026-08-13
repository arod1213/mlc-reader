#!/usr/bin/env bash
set -euo pipefail

#DATA_PATH="./mlc-sample"
#DATA_PATH="./Sample Data"
DATA_PATH="./mlc-bwarm"

export SQLITE_TMPDIR="./sqlite-tmp"
export TMPDIR=$SQLITE_TMPDIR
export RUST_LOG=debug

cargo build --release
BIN="./target/release/mlc-reader"

$BIN save -p "$DATA_PATH"
$BIN migrate -p "$DATA_PATH"
$BIN trim # --vacuum
$BIN index-search
$BIN enrich -m writer > /dev/null
$BIN enrich -m publisher > /dev/null
$BIN enrich -m share > /dev/null 
$BIN update -p './updates.txt'
