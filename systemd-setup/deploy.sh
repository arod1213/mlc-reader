#!/usr/bin/env bash
set -euo pipefail

build_dir="$(mktemp -d)"
trap 'rm -rf "$build_dir"' EXIT

# Install Binaries
go build -o "$build_dir/api" ./api
go build -o "$build_dir/nightjob" ./cmd/night-job

sudo install -d -o root -g root -m 0755 /opt/apg-snowflake

sudo install -o root -g root -m 0755 \
    "$build_dir/api" \
    /opt/apg-snowflake/api

sudo install -o root -g root -m 0755 \
    "$build_dir/nightjob" \
    /opt/apg-snowflake/nightjob

# Setup Environment
sudo install -d -o root -g root -m 0755 /var/lib/apg-snowflake
sudo install -d -o root -g root -m 0755 /etc/apg-snowflake
sudo install -o root -g root -m 0600 .env /etc/apg-snowflake/environment
 
# Setup Systemd Files
sudo install -m 0644 ./systemd-setup/*.service ./systemd-setup/*.timer /etc/systemd/system/
sudo systemctl daemon-reload
 
