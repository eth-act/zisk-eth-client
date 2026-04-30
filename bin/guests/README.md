# Guest Benchmark Scripts

Scripts for benchmarking and comparing the two reth-based guest programs:

| Crate | Description |
|---|---|
| `stateless-validator-reth-baseline` | Baseline build (standard revm) |
| `stateless-validator-u256-by-conditional-compilation` | Switchable U256 backend via conditional compilation |

---

## Setup

Create the virtual environment once:

```bash
cd bin/guests
python3 -m venv .venv
.venv/bin/pip install -r requirements.txt
```

---

## Scripts

### `bench-compare.py` — full build + benchmark + comparison

Builds both guest ELFs, runs `bench.py` for each against all inputs in `reth-inputs/`,
and produces a combined comparison CSV.

```bash
cd bin/guests
.venv/bin/python bench-compare.py
# custom output path:
.venv/bin/python bench-compare.py -o results/comparison.csv
```

Intermediate files written next to the script:
- `bench_baseline.csv` — raw metrics for the baseline
- `bench_u256.csv` — raw metrics for the u256 variant
- `comparison.csv` (default output) — merged file with gain columns

**Comparison CSV columns:**

| Column | Description |
|---|---|
| `filename` | Input `.bin` filename |
| `steps_baseline` | Step count, baseline |
| `steps_u256` | Step count, u256 variant |
| `steps_gain` | `steps_u256 / steps_baseline` — < 1.0 means u256 is faster |
| `total_baseline` | Total metric, baseline |
| `total_u256` | Total metric, u256 variant |
| `total_gain` | `total_u256 / total_baseline` — < 1.0 means u256 is faster |

---

### `bench.py` — run a single ELF against all inputs

Runs `run-stats.sh` for every `.bin` file in `reth-inputs/` and writes a CSV.

```bash
cd bin/guests
.venv/bin/python bench.py -e <path/to/elf> [-o output.csv]
```

**Arguments:**

| Flag | Description |
|---|---|
| `-e` / `--elf` | Path to the ELF file to benchmark (required) |
| `-o` / `--output` | Output CSV path (default: stdout) |

**Output CSV columns:** `filename`, `steps`, `total`

---

### `plot-gains.py` — box plots of gains from comparison CSV

Reads `comparison.csv`, produces a PNG with two box plots (one per metric),
and writes outlier filenames to `outliers.txt`.

```bash
cd bin/guests
.venv/bin/python plot-gains.py
# custom paths:
.venv/bin/python plot-gains.py -i comparison.csv -o gains.png --outliers outliers.txt
```

**Arguments:**

| Flag | Default | Description |
|---|---|---|
| `-i` / `--input` | `comparison.csv` | Input comparison CSV |
| `-o` / `--output` | `gains.png` | Output plot image |
| `--outliers` | `outliers.txt` | Outlier filename list (omitted if no outliers) |

Outliers are detected using the IQR × 1.5 rule, reported in four categories:
`steps_gain low`, `steps_gain high`, `total_gain low`, `total_gain high`.

---

### `run-stats.sh` — run a single ELF on a single input

Low-level helper called by `bench.py`. Runs `ziskemu` (must be on `PATH`) and
prints a JSON object with the two metrics.

```bash
run-stats.sh -e <elf_file> -i <input_file>
# output: {"steps": 12345, "total": 67890}
```
