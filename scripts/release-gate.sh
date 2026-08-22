#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "${BASH_SOURCE[0]}")/.."
source "${HOME}/.cargo/env" 2>/dev/null || true
export PATH="${HOME}/.cargo/bin:${PATH}"

cargo +nightly fmt --all -- --check
CARGO_BUILD_JOBS="${CARGO_BUILD_JOBS:-1}" cargo +nightly build --release --locked -j 1

image=$(find target/release/build/novel_os -type f -name 'novel_os-bios.img' -print -quit)
[[ -n "$image" && -f "$image" ]] || { echo "release gate: BIOS image not found" >&2; exit 1; }
log=$(mktemp)
trace=$(mktemp)
trap 'rm -f "$log" "$trace"' EXIT

set +e
timeout "${QEMU_TIMEOUT_SECONDS:-25}" qemu-system-x86_64 \
  -no-reboot -drive format=raw,file="$image" \
  -serial file:"$log" -display none -monitor none \
  -d int,cpu_reset -D "$trace" >/dev/null 2>&1
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
  'frame recycle invariant: true'
  'heap translation: true'
  'user mapping permissions: true'
  'process address space CR3 isolation: true'
  'process teardown: first=true second=true'
  'ELF loader PT_LOAD+BSS: true'
  'init ELF from ramdisk: fs=true loaded=true bytes='
  'process table: init registered=true current=true state=Running worker=2 state=Ready worker=3 state=Ready'
  'user preemption: contexts=3 timer_frame=true'
  'RING3 transition: init ELF iretq submitted'
  'context switch smoke: true'
  'VFS smoke: true, handles: true, ramdisk: true, global=true'
  'capability table: namespace=true opaque=true generation=true revoked=true'
  'loopback: smoke=true checksum=true backpressure=true'
  'loopback datagram: roundtrip=true ports=1000->2000 checksum=true corruption_rejected=true'
  'block device: ata_pio_sector0=true mbr_partitions=true'
  'persistent fs: novelfs_read=true write_rollback=true'
  'syscall ABI smoke: true'
  'IPC smoke: ports=true generation=true ownership=true'
  'SYSCALL port_create: handle='
  'SYSCALL port_send: handle='
  'SYSCALL port_recv: handle='
  'PROCESS EXIT: status=0 owner='
  'revoked_ports='
  'user memory validation: true'
  'ELF parser validation: true'
  'RING3 int80: cpl=3 valid=true'
  'SYSCALL open: handle='
  'SYSCALL read: handle='
  'SYSCALL close: handle='
  'SYSCALL exit: handoff status=0'
  'USER FAULT POLICY: kill=true status=61444'
  'frames=true starvation=false'
  'PROCESS EXIT: status=0 owner='
  'PROCESS EXIT: status=61444 owner='
  'reclaimed=true next=true'
  'reclaimed=true next=false'
  'USER QUEUE: idle=true runnable=0'
  'preemptive scheduler: task0=4 task1=4 switches=9 starvation=false'
  'hardware IRQs online:'
)
for marker in "${required_markers[@]}"; do
  grep -F "$marker" "$log" >/dev/null || {
    cat "$log"
    echo "missing marker: $marker" >&2
    exit 1
  }
done

cat "$log"
echo "release gate: PASS"
