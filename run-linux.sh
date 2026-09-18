#!/usr/bin/env bash
set -Eeuo pipefail
cd "$(dirname "$0")"
cargo run --release -- --config miner.toml mine
