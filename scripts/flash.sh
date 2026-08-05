#!/usr/bin/env bash
# Build, merge and flash the CH32H417EVT with OpenOCD.
set -euo pipefail

cd "$(dirname "$0")/.."
make flash
