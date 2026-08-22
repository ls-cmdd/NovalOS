# Rebuild Baseline — 2026-08-22

بعد فقدان checkout الأصلي، أُعيد إنشاء workspace من source المتبقي، ثم ثُبّت Rust nightly و`x86_64-unknown-none` و`rust-src` و`llvm-tools-preview` و`rustfmt`، وأُعيد بناء boot pipeline من bootloader 0.11.17.

## Build

`cargo +nightly fmt --all -- --check` و`CARGO_BUILD_JOBS=1 cargo +nightly build --release -j 1` ينجحان. bootloader vendor patch يبني nested BIOS stages تسلسليًا، وroot dependency مقيّدة حاليًا إلى feature `bios` فقط؛ UEFI غير منفذ أو مختبر.

## QEMU proof

BIOS/QEMU أقلع image حقيقية ووصل إلى kernel، allocator، ELF parser/loader، VFS، capability namespace smoke، PIT، ثم Ring 3. النتيجة الأخيرة أثبتت:

```text
RING3 int80: cpl=3 valid=true
SYSCALL write: accepted length=14
USER FAULT POLICY: kill=true status=61444 address=Ok(VirtAddr(0x7fe000)) error=PageFaultErrorCode(USER_MODE)
user preemption: switches=0 frames=true starvation=false
PROCESS EXIT: status=61444 owner=1 state=Faulted ... reclaimed=true next=true
PROCESS HANDOFF: from=1 to=2 state=Running cr3=switched active=true
...
PROCESS EXIT: ... owner=3 ... reclaimed=true next=false
USER QUEUE: idle=true runnable=0
```

The QEMU trace contains only the three intended user-mode page faults at the unmapped guard page. No kernel panic, double fault, or triple fault appeared in the final run.

## Important fixes made during reconstruction

The loader PT_LOAD offset was corrected from zero to 120 bytes so user code, not the ELF header, is copied into the code page. Process roots now preserve the bootloader kernel mapping while clearing PML4[0] before mapping each process's user code and stack. TSS RSP0 now uses the bootloader-provided active kernel stack rather than an unmapped private static stack. The user fixture loop was corrected to increment outside the loop reset, allowing it to reach the guard-page fault.

## Scope boundary

This is a real bootable BIOS development prototype, not a complete distribution or release candidate. The reconstructed scheduler currently restarts the next process at ELF entry after reap, uses a shared kernel stack, does not save FPU/SIMD state, and has no UEFI/SMP, persistent block filesystem, physical NIC/DMA path, or production userland.
