#!/usr/bin/env bash
# Secure local adapter for the existing Arkovia SignTransactions utility.
set -Eeuo pipefail
umask 077

: "${ARKOVIA_NODE_HOME:?Set ARKOVIA_NODE_HOME to a local, built Arkovia Blockchain checkout.}"
command -v jq >/dev/null || { echo "jq is required" >&2; exit 1; }
command -v java >/dev/null || { echo "Java is required" >&2; exit 1; }

work_dir="$(mktemp -d)"
trap 'rm -rf "$work_dir"' EXIT
unsigned_file="$work_dir/unsigned.txt"
signed_file="$work_dir/signed.txt"

# The miner sends only an unsigned transaction to standard input.
jq -er '.unsignedTransactionBytes | select(type == "string" and length > 0)' > "$unsigned_file"

# The Java utility prompts on this physical terminal; the secret phrase is never sent to the node.
java -cp "$ARKOVIA_NODE_HOME/classes:$ARKOVIA_NODE_HOME/lib/*:$ARKOVIA_NODE_HOME/conf" \
  nxt.tools.SignTransactions "$unsigned_file" "$signed_file" < /dev/tty >&2

test -s "$signed_file" || { echo "Arkovia signer produced no signed transaction" >&2; exit 1; }
tr -d '\r\n' < "$signed_file"
