#!/usr/bin/env bash
set -euo pipefail

export SQLITE_TMPDIR="./sqlite-tmp"
export TMPDIR=$SQLITE_TMPDIR
export RUST_LOG=debug

mkdir -p $SQLITE_TMPDIR
chmod 700 $SQLITE_TMPDIR

#TSV_STORAGE_PATH="./mlc-sample"
#TSV_STORAGE_PATH="./Sample Data"
TSV_STORAGE_PATH="./mlc-bwarm"

cargo build --release
BIN="./target/release/mlc-reader"

$BIN save -p "$TSV_STORAGE_PATH"
$BIN migrate -p "$TSV_STORAGE_PATH"
$BIN trim # --vacuum
$BIN index-search
$BIN enrich -m writer > /dev/null
$BIN enrich -m publisher > /dev/null
$BIN enrich -m share > /dev/null 
$BIN update -p './updates.txt'
