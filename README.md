# Novel OS

Novel OS is a freestanding Rust operating-system kernel and bootable platform prototype for x86_64. The project contains a BIOS and UEFI boot path, early memory management, ACPI/MADT discovery, APIC routing, protected user-mode execution, system calls, IPC, capability-scoped VFS access, process teardown, and reproducible QEMU verification.

The repository is intentionally conservative about support claims. BIOS and UEFI boot are verified development-platform paths. An experimental SMP build proves AP bootstrap through Rust, AP-local GDT/TSS/IDT installation, and low-memory per-CPU readiness records. The experimental APs stop before scheduler or user-task execution; the default release package remains single-core.

## Repository layout

| Path | Purpose |
| --- | --- |
| `kernel/` | `no_std` Rust kernel binary and architecture-specific code |
| `vendor/bootloader/` | Vendored bootloader source with the project UEFI compatibility patch |
| `ramdisk/` | Small user fixtures used by the boot and system-call tests |
| `scripts/` | Build, boot, regression, and packaging gates |
| `docs/` | Platform status, verification evidence, roadmap, and design notes |
| `dist/` | Generated BIOS and UEFI release artifacts |

## Requirements

A Linux x86_64 development host is required. Install Rust nightly using the toolchain declared in `rust-toolchain.toml`, together with the `x86_64-unknown-none` and `x86_64-unknown-uefi` targets. The verification scripts also require Cargo, QEMU, and OVMF. The project uses a vendored bootloader dependency and does not require network access after dependencies are available locally.

## Build

Build the default BIOS and UEFI images with:

```sh
cargo +nightly build --release --locked -j 1
```

The generated images are produced by the workspace build script. Use the package gate to build, boot, verify, and package both supported default images:

```sh
./scripts/package-release.sh
```

The package gate writes `novel_os-bios.img`, `novel_os-uefi.img`, checksums, and a manifest into `dist/`. The default package deliberately excludes experimental SMP artifacts.

## Verification

Run the complete default validation suite:

```sh
cargo +nightly fmt --all -- --check
cargo +nightly test -p kernel --bin kernel -j 1
./scripts/release-gate.sh
./scripts/uefi-gate.sh
./scripts/package-release.sh
```

The low-level AP transport proof is available separately:

```sh
QEMU_TIMEOUT_SECONDS=30 ./scripts/smp-low-gate.sh
```

The `smp-experimental` feature is for controlled bring-up diagnostics and is not a production SMP scheduler. Its current QEMU proof and limitations are recorded in [`docs/smp-bringup-evidence.md`](docs/smp-bringup-evidence.md).

## Support status

The current verified scope is a BIOS and UEFI development kernel platform with a single-core default runtime and a gated AP bootstrap experiment. General persistent filesystems, dynamic SMP run queues, FPU/XSAVE context management, PCI/DMA device drivers, network hardware, production userland, an installer, and a package manager are not yet implemented as release features.

Read [`docs/platform-status.md`](docs/platform-status.md), [`docs/verification.md`](docs/verification.md), and [`docs/execution-status.md`](docs/execution-status.md) before relying on any feature. Detailed reproducibility records are in [`docs/final-verification-evidence.md`](docs/final-verification-evidence.md).

## License

Novel OS is distributed under the MIT License. See [`LICENSE`](LICENSE).

## Contributions

Contributions are welcome when they include a clear design rationale, focused changes, tests appropriate to the affected layer, and an update to the support documentation when behavior or platform scope changes. Do not enable experimental SMP paths in the default release package without a reproducible QEMU proof and a documented concurrency audit.
