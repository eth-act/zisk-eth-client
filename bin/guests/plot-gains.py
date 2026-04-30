#!/usr/bin/env python3
"""
Read comparison.csv, plot box plots of steps_gain and total_gain,
and write outlier filenames to outliers.txt.

Usage:
  python3 plot-gains.py [-i <comparison.csv>] [-o <output.png>] [--outliers <outliers.txt>]
"""

import argparse
import csv
import sys
from pathlib import Path

import matplotlib
matplotlib.use("Agg")
import matplotlib.pyplot as plt
import numpy as np

SCRIPT_DIR = Path(__file__).parent


def load(csv_path: Path) -> tuple[list[str], list[float], list[float]]:
    filenames, steps_gains, total_gains = [], [], []
    with open(csv_path, newline="") as f:
        for row in csv.DictReader(f):
            try:
                steps_gains.append(float(row["steps_gain"]))
                total_gains.append(float(row["total_gain"]))
                filenames.append(row["filename"])
            except (ValueError, KeyError):
                pass
    return filenames, steps_gains, total_gains


def outliers_iqr(values: list[float], filenames: list[str]) -> dict[str, list[str]]:
    a = np.array(values)
    q1, q3 = np.percentile(a, 25), np.percentile(a, 75)
    iqr = q3 - q1
    lo, hi = q1 - 1.5 * iqr, q3 + 1.5 * iqr
    return {
        "low":  [filenames[i] for i, v in enumerate(values) if v < lo],
        "high": [filenames[i] for i, v in enumerate(values) if v > hi],
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("-i", "--input",    type=Path, default=SCRIPT_DIR / "comparison.csv")
    parser.add_argument("-o", "--output",   type=Path, default=SCRIPT_DIR / "gains.png")
    parser.add_argument("--outliers",       type=Path, default=SCRIPT_DIR / "outliers.txt")
    args = parser.parse_args()

    if not args.input.exists():
        sys.exit(f"File not found: {args.input}")

    filenames, steps_gains, total_gains = load(args.input)
    if not filenames:
        sys.exit("No valid rows found in CSV.")

    # Invert so the ratio is baseline / u256 (> 1.0 = u256 faster),
    # consistent with the isolated benchmark plots (ruint / accel).
    steps_gains = [1 / v for v in steps_gains]
    total_gains = [1 / v for v in total_gains]

    # ── box plots ─────────────────────────────────────────────────────────
    fig, axes = plt.subplots(1, 2, figsize=(10, 6))

    for ax, values, label in [
        (axes[0], steps_gains, "steps_gain"),
        (axes[1], total_gains, "total_gain"),
    ]:
        ax.boxplot(values, tick_labels=[label])
        ax.set_title(label)
        ax.set_ylabel("baseline / u256")
        ax.axhline(1.0, color="gray", linestyle="--", linewidth=0.8, label="1.0 (no change)")
        ax.legend(fontsize=8)

    fig.suptitle("Gain: baseline / u256  (> 1.0 = u256 faster)")
    fig.tight_layout()
    fig.savefig(args.output, dpi=150)
    print(f"Plot saved to {args.output}")

    # ── outliers ──────────────────────────────────────────────────────────
    steps_out = outliers_iqr(steps_gains, filenames)
    total_out = outliers_iqr(total_gains, filenames)

    lines = []
    for section, names in [
        ("[steps_gain low]",  steps_out["low"]),
        ("[steps_gain high]", steps_out["high"]),
        ("[total_gain low]",  total_out["low"]),
        ("[total_gain high]", total_out["high"]),
    ]:
        if names:
            lines.append(section)
            lines.extend(f"  {n}" for n in names)

    if lines:
        args.outliers.write_text("\n".join(lines) + "\n")
        print(f"Outliers written to {args.outliers}")
    else:
        if args.outliers.exists():
            args.outliers.unlink()
        print("No outliers detected.")


if __name__ == "__main__":
    main()
