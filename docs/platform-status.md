# Platform Status

## BIOS

BIOS is the currently supported and verified platform. The build uses bootloader 0.11.17 with serialized vendor stages, produces a raw BIOS image, and passes the QEMU release gate with Ring3, PIT preemption, IPC, NovelFS, Exit/Fault/reap, and idle evidence.

## UEFI

The required `x86_64-unknown-uefi` Rust target was installed and the build pipeline was probed against `UefiBoot` from bootloader 0.11.17. The first UEFI probe exposed an unresolved `wcslen` symbol in `bootloader-x86_64-uefi`. A vendored COFF shim now resolves that dependency during the nested UEFI bootloader build. The resulting `novel_os-uefi.img` boots under OVMF and passes the independent UEFI gate. UEFI is supported for the currently verified kernel/platform scope; UEFI storage packaging still does not include the BIOS-only appended NovelFS partition.

## First UEFI boot proof

With the local COFF `wcslen` shim, the project produced `novel_os-uefi.img` and booted it under OVMF. The UEFI serial log reached `bootloader API 0.11.17`, memory setup, Ring3, VFS/IPC syscalls, saved-context handoff, owner Exit/Fault/reap, and idle halt without a kernel fault. The UEFI image does not currently carry the BIOS-only NovelFS appended partition, so ATA/NovelFS markers are expected to be false on this path until UEFI storage packaging is added.

## ACPI/MADT and APIC routing proof

The kernel now parses and validates RSDP, RSDT/XSDT headers and checksums, locates MADT, enumerates enabled processor IDs and IOAPIC records, and reports their physical addresses. The parser has four host-side malformed/valid table tests. BIOS QEMU evidence with `-smp 2` reports `ACPI: rsdp=true xsdt=false madt=true cpus=2 ioapics=1 lapic=0xfee00000 ioapic=0xfec00000`; OVMF reports the corresponding XSDT path with one CPU.

LAPIC and IOAPIC routing is implemented and exercised on both BIOS and UEFI paths. The LAPIC timer replaces PIT only after LAPIC/IOAPIC initialization succeeds; otherwise the legacy PIC/PIT fallback remains active. BIOS and UEFI gates both passed after this change. The current code does not claim that SMP is operational.

## SMP bootstrap status: stage 3 proven, scheduling not supported

A real low-memory AP trampoline, AP stack allocation, LAPIC INIT/SIPI sequencing, explicit AP CR3 coverage, AP-local GDT/TSS loaded from the bootstrap stack, a permanent low IDT frame, and BSP/AP low-memory per-CPU records are implemented. The records are selected through `IA32_GS_BASE`, expose interrupt nesting/deferred work/current process metadata, and the AP timer path is quarantined from BSP scheduler state. The reproducible `scripts/smp-low-gate.sh` builds the low-handshake kernel directly and passes its ELF explicitly to the image builder; QEMU `-machine q35 -smp 2` reports `SMP: discovered=2 attempted=1 online=2 trampoline=true`. The separate `smp-experimental` artifact reaches Rust `ap_entry`, records stage 2 on entry and stage 3 after GDT/TSS/IDT, and continues through IRQ, syscall, saved-context preemption, process handoff, and idle markers without an AP fault.

This proves an AP bootstrap context and partial shared-state hardening, not a multicore kernel. Process roots and user context registry are locked and user-process transitions use CAS, but allocator mutation, page-table mutation, scheduler queues, VFS/IPC policy, and interrupt-context ownership still require a full audit. APs therefore still stop in a controlled halt loop. The normal default path remains single-core and reports `SMP: discovered=2 attempted=0 online=1 trampoline=false`; experimental SMP is excluded from release packaging.

Current verified scope is therefore: BIOS boot with ACPI/MADT and APIC routing, UEFI boot under OVMF with ACPI/MADT and APIC routing, single-core kernel/userland execution, and an explicitly gated AP bootstrap proof through stage 3. The release package must continue to contain only the default BIOS and UEFI images until SMP shared-state safety is proven. Detailed evidence is recorded in `docs/smp-bringup-evidence.md`.
