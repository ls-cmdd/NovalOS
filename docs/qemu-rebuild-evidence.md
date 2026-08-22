# QEMU Rebuild Evidence — 2026-08-22

## Command

```text
cargo +nightly fmt --all
CARGO_BUILD_JOBS=1 cargo +nightly build --release -j 1
timeout 25 qemu-system-x86_64 -no-reboot -drive format=raw,file=dist/novel_os-bios.img -serial file=build/qemu.serial -display none -monitor none -d int,cpu_reset -D build/qemu.trace
```

The build returned status 0. QEMU was intentionally stopped by the timeout after the kernel entered its idle halt path, so status 124 is expected for this prototype run.

## Observed semantic markers

```text
VFS smoke: true, handles: true, ramdisk: true, global=true
capability table: namespace=true opaque=true generation=true revoked=true
process table: init registered=true current=true state=Running worker=2 state=Ready worker=3 state=Ready
RING3 int80: cpl=3 valid=true
SYSCALL write: accepted length=14
SYSCALL open: handle=1
SYSCALL read: handle=1 bytes=14
SYSCALL write: accepted length=14
SYSCALL close: handle=1 closed=true
SYSCALL exit: handoff status=0
PROCESS EXIT: status=0 owner=3 state=Exited ... reclaimed=true next=true
PROCESS HANDOFF: from=3 to=1 state=Running cr3=switched active=true
USER FAULT POLICY: kill=true status=61444 ... error=PageFaultErrorCode(USER_MODE)
PROCESS EXIT: status=61444 owner=2 state=Faulted ... reclaimed=true next=true
PROCESS HANDOFF: from=2 to=1 state=Running cr3=switched active=true
PROCESS EXIT: status=61444 owner=1 state=Faulted ... reclaimed=true next=false
USER QUEUE: idle=true runnable=0
```

The run also emitted handles `4194305`, `2`, and `3` in later namespaces, demonstrating that handles are opaque namespace/generation values rather than a global owner-number check. PIT preemption reported `switches=2`, then `5` as the run progressed; exact values are timing-dependent.

QEMU interrupt trace showed only the intended user-mode page faults (`v=0e`, CPL3, CR2=`0x7fe000`). No kernel panic, double fault, or triple fault was observed.

## Current limitation

After an `Exit` or user fault, the reaper starts the next process from its ELF entry rather than returning directly to its preserved saved frame. Timer preemption itself saves/restores GP registers and the iret frame while processes remain runnable. This distinction is documented and must be resolved before calling the scheduler general-purpose.

## Subsequent verified extensions

A later QEMU run verified a real `exit.elf` user image in owner 3 and showed `SYSCALL exit: handoff status=0`, followed by `PROCESS EXIT ... state=Exited`. The reaper was then changed to resume the saved context of the next process instead of restarting from ELF entry. After correcting the assembly restore order so `%rdi` is loaded last, QEMU verified `resume=saved` handoffs, continued filesystem syscalls, and final fault/reap/idle without a kernel fault.

Per-process kernel stacks were added to the TSS and selected on initial entry, timer handoff, and reap handoff. The latest QEMU run remained free of kernel panic, double fault, and triple fault.

An ATA PIO primary-master probe was added and QEMU reported:

```text
block device: ata_pio_sector0=true
```

This proves a real sector-0 read through the emulated ATA controller. It does not yet constitute a persistent filesystem implementation; partition parsing, caching, journaling, and a mountable filesystem remain out of scope.

## Frame ownership and hardware probe extension

The latest run also reported `frame recycle invariant: true` and `process teardown: first=true second=true` after destroy/reap began returning root, code, and stack frames to the bounded allocator recycle pool. The same run reported `block device: ata_pio_sector0=true`, three user filesystem syscall sequences, a real owner-3 Exit, saved-context handoffs, owner-2/owner-1 user faults, and final idle halt. The QEMU fault trace contained only the intended CPL3 guard-page faults.

## MBR partition boundary

A subsequent QEMU run reported `block device: ata_pio_sector0=true mbr_partitions=true`. The kernel read sector 0 through ATA PIO, validated the `0x55AA` signature, and parsed four MBR entry slots, accepting non-empty partition metadata. The run retained successful user open/read/write/close, owner-3 Exit, saved handoff, fault/reap, and idle markers with no kernel fault. This is a storage boundary only; it is not yet a mountable persistent filesystem.

## NovelFS read-only persistent boundary

The build pipeline now appends an 8-sector third MBR partition of type `0xDA` to the BIOS image. It writes a deterministic `NOVELFS1` header, one fixed directory entry, and the `init.txt` data sector. The kernel mounts the partition by reading MBR sector 0 and then reads the directory and data sectors through ATA PIO. QEMU reported:

```text
block device: ata_pio_sector0=true mbr_partitions=true
persistent fs: novelfs_read=true write_rollback=true
```

The updated release gate passed with the same user filesystem, Exit, saved-context, fault/reap, and idle markers. NovelFS now supports a fixed-layout write/read/rollback probe, but it is not yet a general filesystem: journaling, cache, fsck, recovery, directory mutation, concurrent writers, and a general mount VFS remain unimplemented.

## IPC capability extension

The user fixture now executes `port_create`, `port_send`, and `port_recv` through int80. QEMU reported `IPC smoke: ports=true generation=true ownership=true`, three user-level port syscall markers, and `revoked_ports=1` for owners 1 and 2 during reap. Port handles use slot generations and owner checks; stale handles are rejected after owner revocation. The run still completed the Exit/fault/reap/saved-resume/idle sequence without a kernel fault.
