#!/usr/bin/env python3
"""
Merge the CH32H417 V3F waker and V5F application binaries into a single
flash image suitable for OpenOCD.

The V3F image is padded to 0x10000 bytes so that the V5F image starts at
flash offset 0x10000 (physical alias of 0x08010000).
"""

import argparse
import sys
from pathlib import Path

V5F_OFFSET = 0x10000
FILL_BYTE = 0xFF


def merge(v3f_path: Path, v5f_path: Path, out_path: Path) -> None:
    v3f = v3f_path.read_bytes()
    v5f = v5f_path.read_bytes()

    if len(v3f) > V5F_OFFSET:
        print(f"Error: V3F binary is {len(v3f)} bytes, exceeds {V5F_OFFSET:#x}",
              file=sys.stderr)
        sys.exit(1)

    merged = bytearray([FILL_BYTE]) * V5F_OFFSET
    merged[:len(v3f)] = v3f
    merged += v5f

    out_path.write_bytes(merged)
    print(f"Created {out_path}: {len(merged)} bytes")
    print(f"  V3F: 0x00000000 - {len(v3f) - 1:#x}")
    print(f"  V5F: {V5F_OFFSET:#x} - {len(merged) - 1:#x}")


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Merge CH32H417 V3F waker and V5F app binaries")
    parser.add_argument("v3f", type=Path, help="V3F waker binary")
    parser.add_argument("v5f", type=Path, help="V5F application binary")
    parser.add_argument("out", type=Path, help="output merged binary")
    args = parser.parse_args()

    for path in (args.v3f, args.v5f):
        if not path.is_file():
            print(f"Error: {path} not found", file=sys.stderr)
            return 1

    merge(args.v3f, args.v5f, args.out)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
