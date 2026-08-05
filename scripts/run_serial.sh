#!/usr/bin/env bash
# Open a serial terminal on the V5F console port.
set -euo pipefail

cd "$(dirname "$0")/.."
PORT="${1:-/dev/ttyACM0}"
BAUD="${2:-115200}"

echo "Opening serial terminal on $PORT @ $BAUD baud..."
echo "Use Ctrl-a k to kill screen, or Ctrl-c if using minicom."

if command -v screen >/dev/null 2>&1; then
    screen "$PORT" "$BAUD"
elif command -v minicom >/dev/null 2>&1; then
    minicom -D "$PORT" -b "$BAUD"
else
    echo "Error: neither 'screen' nor 'minicom' is installed." >&2
    exit 1
fi
