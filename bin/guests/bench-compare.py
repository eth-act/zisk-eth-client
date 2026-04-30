#!/usr/bin/env python3
"""
Build both guest ELFs, run bench.py for each, and produce a comparison CSV.

Usage:
  python3 bench-compare.py [-o <output.csv>]

  -o / --output  Combined output CSV path (default: comparison.csv next to this script)

The comparison CSV columns:
  filename,
  steps_baseline, steps_u256, steps_gain,
  total_baseline, total_u256, total_gain

*_gain = u256 / baseline  (lower is better for the u256 variant)
"""

import argparse
import csv
import subprocess
import sys
from pathlib import Path

SCRIPT_DIR = Path(__file__).parent
GUESTS_DIR = SCRIPT_DIR  # same directory

BASELINE_DIR = GUESTS_DIR / "stateless-validator-reth-baseline"
U256_DIR     = GUESTS_DIR / "stateless-validator-u256-by-conditional-compilation"
BASELINE_ELF = BASELINE_DIR / "target" / "riscv64ima-zisk-zkvm-elf" / "release" / "zec-reth"
U256_ELF     = U256_DIR    / "target" / "riscv64ima-zisk-zkvm-elf" / "release" / "zec-reth"

BENCH_PY = SCRIPT_DIR / "bench.py"
U256_ZISKEMU_PATH = "/projects/EF/zisk-repos/zisk/target/release"


def build(crate_dir: Path):
    print(f"\n=== Building {crate_dir.name} ===", flush=True)
    print("  $ cargo-zisk build --release", flush=True)
    result = subprocess.run(["cargo-zisk", "build", "--release"], cwd=crate_dir)
    if result.returncode != 0:
        sys.exit(f"Build failed in {crate_dir.name}")


def run_bench(elf: Path, csv_out: Path, extra_path: str | None = None):
    print(f"\n=== Benchmarking {elf.parent.parent.name} ===", flush=True)
    env = None
    if extra_path:
        import os
        env = os.environ.copy()
        env["PATH"] = f"{extra_path}:{env['PATH']}"
    result = subprocess.run(
        [sys.executable, str(BENCH_PY), "-e", str(elf), "-o", str(csv_out)],
        env=env,
    )
    if result.returncode != 0:
        sys.exit(f"bench.py failed for {elf}")


def read_csv(path: Path) -> dict[str, dict]:
    rows = {}
    with open(path, newline="") as f:
        for row in csv.DictReader(f):
            rows[row["filename"]] = row
    return rows


def main():
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("-o", "--output", type=Path, default=SCRIPT_DIR / "comparison.csv")
    args = parser.parse_args()

    baseline_csv = SCRIPT_DIR / "bench_baseline.csv"
    u256_csv     = SCRIPT_DIR / "bench_u256.csv"

    build(BASELINE_DIR)
    build(U256_DIR)

    run_bench(BASELINE_ELF, baseline_csv)
    run_bench(U256_ELF,     u256_csv, extra_path=U256_ZISKEMU_PATH)

    baseline = read_csv(baseline_csv)
    u256     = read_csv(u256_csv)

    filenames = sorted(set(baseline) | set(u256))

    with open(args.output, "w", newline="") as f:
        writer = csv.DictWriter(f, fieldnames=[
            "filename",
            "steps_baseline", "steps_u256", "steps_gain",
            "total_baseline", "total_u256", "total_gain",
        ])
        writer.writeheader()
        for name in filenames:
            b = baseline.get(name, {})
            u = u256.get(name, {})
            def gain(b_val, u_val):
                try:
                    return f"{int(u_val) / int(b_val):.6f}"
                except (ValueError, ZeroDivisionError):
                    return ""
            writer.writerow({
                "filename":       name,
                "steps_baseline": b.get("steps", ""),
                "steps_u256":     u.get("steps", ""),
                "steps_gain":     gain(b.get("steps", ""), u.get("steps", "")),
                "total_baseline": b.get("total", ""),
                "total_u256":     u.get("total", ""),
                "total_gain":     gain(b.get("total", ""), u.get("total", "")),
            })

    print(f"\nComparison written to {args.output}", flush=True)


if __name__ == "__main__":
    main()
