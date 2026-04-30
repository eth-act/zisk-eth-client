#!/usr/bin/env python3
"""
Run run-stats.sh for every .bin file in reth-inputs/ and write a CSV.

Usage:
  python3 bench.py -e <elf_file> [-o <output.csv>]

  -e / --elf     Path to the ELF file (required)
  -o / --output  Output CSV path (default: stdout)
"""

import argparse
import csv
import json
import subprocess
import sys
from pathlib import Path

SCRIPT_DIR = Path(__file__).parent
RUN_STATS = SCRIPT_DIR / "run-stats.sh"
INPUTS_DIR = SCRIPT_DIR / "reth-inputs"


def main():
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("-e", "--elf", required=True, type=Path)
    parser.add_argument("-o", "--output", type=Path, default=None)
    args = parser.parse_args()

    elf = args.elf.resolve()
    if not elf.exists():
        sys.exit(f"ELF not found: {elf}")

    input_files = sorted(INPUTS_DIR.glob("*.bin"))
    if not input_files:
        sys.exit(f"No .bin files found in {INPUTS_DIR}")

    print(f"ELF:    {elf}", file=sys.stderr)
    print(f"Inputs: {len(input_files)} files", file=sys.stderr)

    out = open(args.output, "w", newline="") if args.output else sys.stdout
    writer = csv.DictWriter(out, fieldnames=["filename", "steps", "total"])
    writer.writeheader()

    errors = []
    for i, f in enumerate(input_files, 1):
        result = subprocess.run(
            ["bash", str(RUN_STATS), "-e", str(elf), "-i", str(f)],
            capture_output=True, text=True,
        )
        if result.returncode != 0:
            err = result.stderr.strip()
            print(f"[{i}/{len(input_files)}] {f.name}: ERROR: {err}", file=sys.stderr)
            errors.append(f"{f.name}: {err}")
            writer.writerow({"filename": f.name, "steps": "", "total": ""})
            continue

        data = json.loads(result.stdout)
        writer.writerow({"filename": f.name, "steps": data["steps"], "total": data["total"]})
        print(f"[{i}/{len(input_files)}] {f.name}: steps={data['steps']} total={data['total']}", file=sys.stderr)

    if args.output:
        out.close()
        print(f"Written to {args.output}", file=sys.stderr)

    if errors:
        print(f"\n{len(errors)} error(s):", file=sys.stderr)
        for e in errors:
            print(f"  {e}", file=sys.stderr)
        sys.exit(1)


if __name__ == "__main__":
    main()
