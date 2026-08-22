#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "${BASH_SOURCE[0]}")/.."
source "${HOME}/.cargo/env" 2>/dev/null || true
export PATH="${HOME}/.cargo/bin:${PATH}"

OVMF_CODE_PATH="${OVMF_CODE_PATH:-/usr/share/OVMF/OVMF_CODE_4M.fd}"
OVMF_VARS_PATH="${OVMF_VARS_PATH:-/usr/share/OVMF/OVMF_VARS_4M.fd}"
[[ -r "$OVMF_CODE_PATH" ]] || { echo "UEFI gate: OVMF code not found: $OVMF_CODE_PATH" >&2; exit 1; }
[[ -r "$OVMF_VARS_PATH" ]] || { echo "UEFI gate: OVMF vars template not found: $OVMF_VARS_PATH" >&2; exit 1; }

cargo +nightly fmt --all -- --check
CARGO_BUILD_JOBS="${CARGO_BUILD_JOBS:-1}" cargo +nightly build --release --locked -j 1

image=$(find target/release/build/novel_os -type f -name 'novel_os-uefi.img' -print -quit)
[[ -n "$image" && -f "$image" ]] || { echo "UEFI gate: UEFI image not found" >&2; exit 1; }

log=$(mktemp)
trace=$(mktemp)
vars=$(mktemp)
trap 'rm -f "$log" "$trace" "$vars"' EXIT
cp "$OVMF_VARS_PATH" "$vars"

set +e
timeout "${QEMU_TIMEOUT_SECONDS:-25}" qemu-system-x86_64 \
  -machine q35 -m "${QEMU_MEMORY:-256M}" \
  -drive if=pflash,format=raw,readonly=on,file="$OVMF_CODE_PATH" \
  -drive if=pflash,format=raw,file="$vars" \
  -drive format=raw,file="$image" \
  -serial file:"$log" -display none -monitor none \
  -no-reboot -d int,cpu_reset -D "$trace" >/dev/null 2>&1
status=$?
set -e

if [[ "$status" -ne 0 && "$status" -ne 124 ]]; then
  cat "$log"
  exit 1
fi
if grep -Eq 'Triple fault|PANIC|KERNEL ALLOCATION ERROR|EXCEPTION: PAGE FAULT|DOUBLE FAULT|unable to map' "$log"; then
  cat "$log"
  exit 1
fi

required_markers=(
  'bootloader API: ApiVersion'
  'physical memory offset:'
  'ACPI: rsdp=true'
  'madt=true cpus='
  'frame recycle invariant: true'
  'heap translation: true'
  'process address space CR3 isolation: true'
  'ELF loader PT_LOAD+BSS: true'
  'RING3 transition: init ELF iretq submitted'
  'RING3 int80: cpl=3 valid=true'
  'context switch smoke: true'
  'VFS smoke: true, handles: true, ramdisk: true, global=true'
  'IPC smoke: ports=true generation=true ownership=true'
  'SYSCALL port_create: handle='
  'SYSCALL port_send: handle='
  'SYSCALL port_recv: handle='
  'SYSCALL exit: handoff status=0'
  'PROCESS EXIT: status=0 owner='
  'USER FAULT POLICY: kill=true status=61444'
  'USER QUEUE: idle=true runnable=0'
)
for marker in "${required_markers[@]}"; do
  grep -F "$marker" "$log" >/dev/null || {
    cat "$log"
    echo "missing marker: $marker" >&2
    exit 1
  }
done

cat "$log"
echo "UEFI gate: PASS"
