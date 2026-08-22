# Bootloader Sources and Local Patches

Novel OS uses bootloader 0.11.17 from the Rust OSDev project. The repository vendors the dependency under `vendor/bootloader/` so that builds remain reproducible and do not depend on a mutable checkout of an external source tree.

The upstream basic example documents the workspace and build-script arrangement used by this project:

1. [`Cargo.toml`](https://raw.githubusercontent.com/rust-osdev/bootloader/v0.11.17/examples/basic/Cargo.toml) shows the kernel artifact dependency and the `x86_64-unknown-none` target.
2. [`build.rs`](https://raw.githubusercontent.com/rust-osdev/bootloader/v0.11.17/examples/basic/build.rs) shows BIOS disk-image creation from the kernel artifact.
3. [`.cargo/config.toml`](https://raw.githubusercontent.com/rust-osdev/bootloader/v0.11.17/examples/basic/.cargo/config.toml) documents the unstable binary-dependency configuration.
4. The [disk-image documentation](https://raw.githubusercontent.com/rust-osdev/bootloader/v0.11.17/docs/create-disk-image.md) describes the build-script architecture.

The vendored copy contains the local compatibility changes required by this project. BIOS and UEFI boot images are built by the root build script; UEFI is verified independently under OVMF. The local build path serializes nested bootloader stages to keep resource usage deterministic. Any local patch that changes boot behavior must be documented and covered by the BIOS and UEFI gates.
