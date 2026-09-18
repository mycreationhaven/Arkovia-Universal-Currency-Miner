# Arkovia Universal Currency Miner

A Rust CPU miner for Scrypt (`algorithm = 5`) currencies issued through the Arkovia Monetary System. It performs the memory-hard Scrypt work locally, uses the exact five-field little-endian work format required by Arkovia, refreshes targets through the node API, and never transmits a secret phrase.

## MVP capabilities

- Configurable node, currency code/ID, mint amount, wallet account/public key, CPU threads, and fee.
- Protocol-compatible Scrypt work: `N=1024, r=1, p=1`, 32-byte output.
- Uses `getCurrency`, `getMintingTarget`, `currencyMint`, and `broadcastTransaction`.
- Enforces the 0.01 ARKOS minimum fee (`1,000,000 NQT`).
- Refreshes stale targets on a configurable timer and displays a live hash rate and attempts counter.
- `prepare` mode saves unsigned mint transactions locally. `broadcast` mode calls a **local** signer command; secret phrases never leave the local device.
- Matrix-style terminal display and Windows/Linux launch scripts.

## First testnet run

1. Install the stable [Rust toolchain](https://rustup.rs/).
2. Copy `miner.example.toml` to `miner.toml` and fill in your testnet currency, account RS address, numeric account ID, and node URL.
3. Run `cargo test`, then `cargo run --release -- --config miner.toml status`.
4. Start mining with `cargo run --release -- --config miner.toml mine`, or use `run-linux.sh` / `run-windows.bat`.

The first release deliberately defaults to `submit_mode = "prepare"`. This lets you inspect the unsigned mint transaction and keeps the secret phrase out of configuration files and network requests. A local signing adapter will be included before switching production use to broadcast mode.

## Security

Do not put a secret phrase in `miner.toml`, a command line, a batch file, or an issue. Use a testnet account for early validation.
