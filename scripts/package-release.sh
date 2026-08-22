#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
cd "$ROOT_DIR"
source "${HOME}/.cargo/env" 2>/dev/null || true
export PATH="${HOME}/.cargo/bin:${PATH}"

cargo +nightly fmt --all -- --check
CARGO_BUILD_JOBS="${CARGO_BUILD_JOBS:-1}" cargo +nightly build --release --locked -j 1
./scripts/release-gate.sh
./scripts/uefi-gate.sh

mkdir -p dist
BIOS_IMAGE=$(find target/release/build/novel_os -type f -name 'novel_os-bios.img' -print -quit)
UEFI_IMAGE=$(find target/release/build/novel_os -type f -name 'novel_os-uefi.img' -print -quit)
if [[ -z "$BIOS_IMAGE" || ! -f "$BIOS_IMAGE" ]]; then
  echo "missing BIOS image under target/release/build/novel_os" >&2
  exit 1
fi
if [[ -z "$UEFI_IMAGE" || ! -f "$UEFI_IMAGE" ]]; then
  echo "missing UEFI image under target/release/build/novel_os" >&2
  exit 1
fi

cp "$BIOS_IMAGE" dist/novel_os-bios.img
cp "$UEFI_IMAGE" dist/novel_os-uefi.img
sha256sum dist/novel_os-bios.img > dist/novel_os-bios.img.sha256
sha256sum dist/novel_os-uefi.img > dist/novel_os-uefi.img.sha256
sha256sum Cargo.lock > dist/Cargo.lock.sha256
{
  echo "artifacts=novel_os-bios.img,novel_os-uefi.img"
  echo "bios_sha256=$(cut -d' ' -f1 dist/novel_os-bios.img.sha256)"
  echo "uefi_sha256=$(cut -d' ' -f1 dist/novel_os-uefi.img.sha256)"
  echo "cargo_lock_sha256=$(cut -d' ' -f1 dist/Cargo.lock.sha256)"
  echo "git_commit=$(git rev-parse HEAD 2>/dev/null || echo unavailable)"
  echo "git_tag=$(git describe --tags --always --dirty 2>/dev/null || echo untagged)"
  echo "rustc=$(rustc +nightly --version)"
  echo "scope=BIOS and UEFI development kernel platform milestone; OVMF verified"
  echo "excluded=SMP,NIC,DMA,general-persistent-filesystem,dynamic-run-queue,FPU-SIMD,production-userland,CI-external"
} > dist/manifest.txt
cat dist/manifest.txt
