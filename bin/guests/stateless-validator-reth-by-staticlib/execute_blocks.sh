#!/bin/bash

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# ZisK zkVM target
RUSTFLAGS='--cfg zisk_hints --cfg zisk_hints_debug' cargo build --release
cargo build --release
cargo-zisk build --release

# Bare-metal RISC-V (riscv64im-unknown-none-elf) target
RISCV_TARGET_SPEC="$SCRIPT_DIR/riscv64im-unknown-none-elf.json"
RUSTFLAGS="-C passes=lower-atomic" cargo +nightly build --release \
  --no-default-features \
  --target "$RISCV_TARGET_SPEC" \
  -Zbuild-std=core,alloc \
  -Zjson-target-spec

RISCV_ELF="$SCRIPT_DIR/target/riscv64im-unknown-none-elf/release/zec-reth"

echo "Execution for host"

for x in inputs/*; do
  ln -sf $(realpath $x) build/input.bin
  # Exit code 61 is the ZisK host runtime's normal exit via zkvm_deinit(); treat it as success.
  cargo run --release || [ $? -eq 61 ]
done

export PATH=/projects/EF/zisk-repos/zisk/target/release:$PATH

echo "execution for RISCV64IM"

for x in inputs/*; do
  ziskemu -e $RISCV_ELF -i $x;
done

echo "execution for ZisK"

for x in inputs/*; do
  ziskemu -e target/elf/riscv64ima-zisk-zkvm-elf/release/zec-reth -i $x;
done
