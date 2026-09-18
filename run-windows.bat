@echo off
cd /d "%~dp0"
cargo run --release -- --config miner.toml mine
