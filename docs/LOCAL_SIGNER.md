# Local signing adapter

The miner deliberately does not implement a secret-phrase prompt or send a secret phrase to an Arkovia node. It creates unsigned transaction bytes and passes them only to a **local** command when `submit_mode = "broadcast"`.

## Contract

The command in `[signer].command` receives one JSON object through standard input:

```json
{
  "unsignedTransactionBytes": "<hex>",
  "transactionJSON": { "...": "..." }
}
```

It must return exactly one value to standard output: the signed transaction bytes as lowercase or uppercase hexadecimal. Diagnostic output must go to standard error. A zero exit status means success.

## Security requirements

- The signer must run on the same computer as the miner.
- It must prompt for the secret phrase without echoing it and must clear it from memory as far as the platform allows.
- Do not put the secret phrase in `miner.toml`, a batch file, an environment variable, process arguments, logs, or a cloud service.
- Do not use a remote shell command, web URL, or node API endpoint as `signer.command`.
- Start on testnet or a deliberately low-value test currency.

## Example configuration

```toml
[miner]
submit_mode = "broadcast"

[signer]
command = "./arkovia-local-signer"
```

On Windows, use a local `.exe` path such as:

```toml
command = "arkovia-local-signer.exe"
```

## Included adapters

This repository includes adapters that call Arkovia's existing offline `nxt.tools.SignTransactions` utility. That utility uses the same transaction and Curve25519 signing code as Arkovia itself.

- Linux: `signer/arkovia-local-signer.sh`
- Windows: `signer/arkovia-local-signer.ps1`

Both adapters receive the unsigned bytes from the miner, create restricted temporary files, prompt locally for the secret phrase, invoke Arkovia's offline signer, print only signed transaction bytes to standard output, and erase the temporary directory afterwards.

They require a **local, built** checkout of [Arkovia Blockchain](https://github.com/mycreationhaven/Arkovia-Blockchain). The checkout must contain `classes`, `lib`, and `conf`. Set `ARKOVIA_NODE_HOME` to that directory.

### Linux configuration

```bash
chmod +x signer/arkovia-local-signer.sh
export ARKOVIA_NODE_HOME=/opt/arkos
```

```toml
[miner]
submit_mode = "broadcast"

[signer]
command = "./signer/arkovia-local-signer.sh"
```

Install `jq` and a Java runtime first. The script reads the phrase from `/dev/tty`, so it prompts visibly even though the miner sends it a JSON payload through standard input.

### Windows configuration

In PowerShell, set the local checkout path for the current session:

```powershell
$env:ARKOVIA_NODE_HOME = "C:\Arkovia-Blockchain"
```

Then set:

```toml
[miner]
submit_mode = "broadcast"

[signer]
command = "powershell -ExecutionPolicy Bypass -File .\signer\arkovia-local-signer.ps1"
```

The PowerShell adapter prompts using a protected input field, then passes the phrase only through a local process pipe to the Arkovia signer.
