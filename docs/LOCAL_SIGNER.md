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

The dedicated signer implementation is the next security-sensitive component. It must use Arkovia's exact Curve25519 transaction-signing implementation and will be validated against known signed transaction vectors before it is enabled for production use.
