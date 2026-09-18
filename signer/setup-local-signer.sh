#!/usr/bin/env bash
# Builds the local Arkovia signing dependency without starting a node.
set -Eeuo pipefail
: "${ARKOVIA_NODE_HOME:?Choose a local destination, for example: export ARKOVIA_NODE_HOME=$HOME/Arkovia-Blockchain}"
command -v git >/dev/null || { echo "Install git first." >&2; exit 1; }
command -v javac >/dev/null || { echo "Install a Java JDK (not only a JRE) first." >&2; exit 1; }

if [ ! -d "$ARKOVIA_NODE_HOME/.git" ]; then
  git clone https://github.com/mycreationhaven/Arkovia-Blockchain.git "$ARKOVIA_NODE_HOME"
fi
cd "$ARKOVIA_NODE_HOME"
./compile.sh --skip-desktop
echo "Offline signer dependency is ready at: $ARKOVIA_NODE_HOME"
