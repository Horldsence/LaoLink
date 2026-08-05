#!/usr/bin/env bash
# Build both V3F waker and V5F application images.
set -euo pipefail

cd "$(dirname "$0")/.."
make build
