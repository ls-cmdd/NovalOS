#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "${BASH_SOURCE[0]}")/.."
export PATH="$HOME/.cargo/bin:$PATH"

QEMU_TIMEOUT_SECONDS="${QEMU_TIMEOUT_SECONDS:-30}"
LOG_DIR="${TMPDIR:-/tmp}/novel-os-smp-low"
rm -rf "$LOG_DIR"
mkdir -p "$LOG_DIR"

cargo +nightly fmt --all -- --check
CARGO_BUILD_JOBS=1 cargo +nightly build --release --locked -p kernel \
  --target x86_64-unknown-none --features smp-low-handshake -j 1 \
  >"$LOG_DIR/kernel-build.log" 2>&1

kernel_artifact="$(pwd)/target/x86_64-unknown-none/release/kernel"
if [[ ! -f "$kernel_artifact" ]]; then
  echo "SMP low gate: low-handshake kernel artifact not found" >&2
  exit 1
fi

NOVEL_KERNEL_ARTIFACT="$kernel_artifact" \
  CARGO_BUILD_JOBS=1 cargo +nightly build --release --locked -j 1 \
  >"$LOG_DIR/image-build.log" 2>&1

image="$(find target/release/build/novel_os -type f -name 'novel_os-bios.img' -printf '%T@ %p\n' \
  | sort -nr | head -1 | cut -d' ' -f2-)"
if [[ -z "$image" || ! -f "$image" ]]; then
  echo "SMP low gate: BIOS image not found" >&2
  exit 1
fi

timeout "$QEMU_TIMEOUT_SECONDS" qemu-system-x86_64 \
  -machine q35 -smp 2 -m 256M \
  -drive "format=raw,file=$image" \
  -serial "file:$LOG_DIR/serial.log" \
  -display none -monitor none -no-reboot \
  >"$LOG_DIR/qemu.stdout" 2>&1 || qemu_status=$?
qemu_status="${qemu_status:-0}"
if [[ "$qemu_status" -ne 0 && "$qemu_status" -ne 124 ]]; then
  echo "SMP low gate: QEMU failed with status $qemu_status" >&2
  cat "$LOG_DIR/qemu.stdout" >&2
  exit 1
fi

if ! grep -Eq 'SMP: discovered=[2-9][0-9]* attempted=[1-9][0-9]* online=2 trampoline=true' "$LOG_DIR/serial.log"; then
  echo "SMP low gate: online proof missing" >&2
  grep -E 'SMP:|PANIC|Triple fault|PAGE FAULT' "$LOG_DIR/serial.log" >&2 || true
  exit 1
fi

if grep -Eq 'Triple fault|PANIC' "$LOG_DIR/serial.log"; then
  echo "SMP low gate: fatal kernel signature found" >&2
  exit 1
fi

grep -E 'ACPI:|APIC routing|SMP:|SYSCALL exit|USER QUEUE' "$LOG_DIR/serial.log"
echo "SMP low gate: PASS"
