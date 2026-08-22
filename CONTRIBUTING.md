# Contributing to Novel OS

Thank you for contributing to Novel OS. The project is a freestanding Rust operating-system kernel, so changes must preserve bootability, memory-safety assumptions, and reproducible verification.

## Development rules

Keep changes focused and explain architectural decisions in the code or in the appropriate document under `docs/`. Assembly, `unsafe`, page-table, interrupt, descriptor, allocator, and context-switch changes must include comments that state the invariant being relied upon. Do not convert an experimental feature into a default release path without a reproducible QEMU or OVMF test.

Do not commit generated build directories, local logs, temporary traces, credentials, private keys, or machine-specific paths. Release artifacts belong in `dist/` only when they were produced by the packaging script and their checksums and manifest are updated together.

## Validation

Before opening a change, run the checks relevant to the affected layer. The standard suite is:

```sh
cargo +nightly fmt --all -- --check
cargo +nightly test -p kernel --bin kernel -j 1
./scripts/release-gate.sh
./scripts/uefi-gate.sh
./scripts/package-release.sh
```

Changes to the AP bootstrap path must also pass:

```sh
QEMU_TIMEOUT_SECONDS=30 ./scripts/smp-low-gate.sh
```

If a test is intentionally not run, state the reason and the remaining risk in the change description. Update `docs/platform-status.md` and the relevant evidence document whenever verified scope changes.

## Pull requests

A pull request should describe the problem, the design, the validation commands and results, and any known limitations. Claims about SMP, persistence, hardware support, or userland must be supported by an executable test rather than a marker or compile-only result.
