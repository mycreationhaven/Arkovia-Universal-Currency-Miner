# Running Arkovia Universal Currency Miner

This guide is for the current Rust-source MVP. Begin with a testnet account or a low-value mintable Scrypt currency. The miner supports Arkovia Monetary System Scrypt currencies only (`algorithm = 5`); it does not mine ARKOS staking blocks.

## Before you start

You need:

- A 64-bit Windows or Linux computer with internet access to an Arkovia node.
- An Arkovia account in RS format and its numeric account ID.
- A mintable Scrypt currency code, such as `MLT` when Meldralite is configured as a mintable Scrypt currency.
- A public key for the same Arkovia account. A public key is safe to place in the configuration file; a secret phrase is not.
- Enough ARKOS to pay the minimum mint transaction fee: `0.01 ARKOS` (`1000000 NQT`).

## Linux

### 1. Install the Rust toolchain

On Ubuntu/Debian, run:

```bash
sudo apt update
sudo apt install -y build-essential pkg-config curl
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
```

Confirm it installed:

```bash
rustc --version
cargo --version
```

### 2. Download and configure the miner

```bash
git clone https://github.com/mycreationhaven/Arkovia-Universal-Currency-Miner.git
cd Arkovia-Universal-Currency-Miner
cp miner.example.toml miner.toml
nano miner.toml
```

Set the `[node]`, `[currency]`, and `[wallet]` values. Example:

```toml
[node]
url = "https://arkovia-node1.mywire.org/nxt"

[currency]
name = "Meldralite"
code = "MLT"
units_per_mint = 1

[wallet]
account_rs = "ARK-XXXX-XXXX-XXXX-XXXXX"
account_id = "YOUR_NUMERIC_ACCOUNT_ID"
public_key = "YOUR_64_CHARACTER_HEX_PUBLIC_KEY"
```

Leave `submit_mode = "prepare"` for your first run. It writes an unsigned transaction locally after a valid solution is found; it does not submit it.

### 3. Check the node and build

```bash
cargo run --release -- --config miner.toml status
cargo build --release
```

The compiled program is `target/release/arkovia-universal-currency-miner`.

### 4. Start mining

```bash
./run-linux.sh
```

Or run the compiled program directly:

```bash
./target/release/arkovia-universal-currency-miner --config miner.toml mine
```

Stop it with `Ctrl+C`. Each active CPU thread uses memory because Scrypt is memory-hard; reduce `threads` in `miner.toml` if the computer becomes slow.

## Windows 10/11 (64-bit)

### 1. Install required tools

Install these two official tools, then restart PowerShell:

- [Git for Windows](https://git-scm.com/download/win)
- [Rustup for Windows](https://rustup.rs/)

Open PowerShell and confirm:

```powershell
rustc --version
cargo --version
git --version
```

### 2. Download and configure the miner

```powershell
git clone https://github.com/mycreationhaven/Arkovia-Universal-Currency-Miner.git
cd Arkovia-Universal-Currency-Miner
Copy-Item miner.example.toml miner.toml
notepad miner.toml
```

Fill in the same `[node]`, `[currency]`, and `[wallet]` details shown in the Linux example. Keep the fee at `1000000` NQT or higher.

### 3. Check the node and build

```powershell
cargo run --release -- --config miner.toml status
cargo build --release
```

The compiled program is `target\release\arkovia-universal-currency-miner.exe`.

### 4. Start mining

Double-click `run-windows.bat`, or run:

```powershell
.\target\release\arkovia-universal-currency-miner.exe --config miner.toml mine
```

Use `Ctrl+C` to stop it. If Windows Defender asks about the self-built executable, review the source and allow it only if you built it from this repository yourself.

## Prepared and broadcast modes

`prepare` is the default and safest mode. It writes `prepared-mint-<timestamp>.json` beside the miner. These files are ignored by Git but should still be kept private.

`broadcast` requires a dedicated local signing adapter. Configure it only after the adapter is installed and validated; see [LOCAL_SIGNER.md](LOCAL_SIGNER.md).

## Optional: enable local signing and broadcast

The included adapters use the offline signing tool included with a local Arkovia Blockchain checkout. They do **not** send your phrase to an Arkovia node.

### Linux

Install the additional packages:

```bash
sudo apt install -y jq default-jre
```

Build or otherwise prepare a local Arkovia Blockchain checkout that contains `classes`, `lib`, and `conf`, then run:

```bash
export ARKOVIA_NODE_HOME=/opt/arkos
chmod +x signer/arkovia-local-signer.sh
```

Set the two `broadcast` settings shown in [LOCAL_SIGNER.md](LOCAL_SIGNER.md). When a solution is found, the terminal prompts for the phrase locally, signs through Arkovia's offline signer, then broadcasts the signed transaction.

### Windows

Install a Java runtime and prepare a local built Arkovia Blockchain checkout. In the same PowerShell session used to launch the miner:

```powershell
$env:ARKOVIA_NODE_HOME = "C:\Arkovia-Blockchain"
```

Set the PowerShell `command` and `broadcast` settings from [LOCAL_SIGNER.md](LOCAL_SIGNER.md). The window prompts for the phrase securely when a solution is found.

## Troubleshooting

| Message | Meaning and action |
|---|---|
| `not a Scrypt minting currency` | The chosen currency is not configured with Scrypt algorithm ID 5. Use the correct currency code. |
| `fee_nqt must be at least 1000000` | Raise the configured fee to at least 0.01 ARKOS. |
| `No wallet.public_key configured` | Add the 64-character public key for the wallet you are mining with. |
| `Connection` error | Check `node.url`, your internet connection, and whether the node is online. |
| Computer becomes slow | Lower `[miner].threads`, such as `threads = 1` or `threads = 2`. |
