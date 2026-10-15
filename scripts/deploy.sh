#!/usr/bin/env bash
set -euo pipefail

build_dir="$(mktemp -d)"
trap 'rm -rf "$build_dir"' EXIT

# Install Binaries
cargo build --release --bin api --bin mlc-reader
install -m 0755 target/release/api "$build_dir/api"
install -m 0755 target/release/mlc-reader "$build_dir/nightjob"

sudo install -d -o root -g root -m 0755 /opt/apg-mlc

sudo install -o root -g root -m 0755 \
    "$build_dir/api" \
    /opt/apg-mlc/api

sudo install -o root -g root -m 0755 \
    "$build_dir/nightjob" \
    /opt/apg-mlc/nightjob

# Setup Environment
sudo install -d -o root -g root -m 0755 /var/lib/apg-mlc
sudo install -d -o root -g root -m 0755 /etc/apg-mlc
sudo install -o root -g root -m 0600 .env /etc/apg-mlc/environment
 
# Setup Systemd Files
sudo install -m 0644 ./systemd-setup/*.service ./systemd-setup/*.timer /etc/systemd/system/
sudo systemctl daemon-reload
