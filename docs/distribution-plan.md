# Novel OS Distribution Plan

## الهدف

الهدف هو نقل Novel OS من **BIOS-only development prototype** إلى توزيعة تشغيل قابلة للتثبيت والإقلاع والتحديث، بنواة Rust حقيقية وuserland موثق. لا تُعتبر المهمة منجزة بمجرد وجود واجهة أو marker؛ يلزم implementation، اختبار سلبي وإيجابي، ونتيجة قابلة لإعادة التشغيل على QEMU أو عتاد مدعوم.

## مصفوفة التنفيذ

| الطبقة | المخرج المطلوب | معيار القبول التنفيذي | الوضع عند baseline `ipc-capability-v1` |
|---|---|---|---|
| Boot | BIOS ثم UEFI loader مع انتقال موحد إلى kernel | صورتا BIOS/UEFI تقلعان وتقدمان memory map وconsole | BIOS منجز جزئيًا؛ UEFI لاحق |
| Platform | ACPI، APIC/IOAPIC، SMP، per-CPU state | CPUs متعددة تعمل دون shared-stack corruption أو lost interrupts | PIC/PIT وper-process stacks مثبتة؛ SMP لاحق |
| Memory | frame ownership، page allocator، user/kernel permissions، reclaim | map/unmap/recycle tests، no duplicate live frames، fault containment | allocator وCR3 roots وrecycle أساسي مثبت؛ ownership الكامل لاحق |
| Scheduler | threads/processes، dynamic run queue، idle، priorities، context state | spawn/exit/wait، fairness، FPU/SIMD preservation، lock/preemption tests | bounded 3-context queue وsaved resume مثبت؛ dynamic/FPU لاحق |
| Syscalls | versioned ABI، copy-in/out، errno، wait/signal/time | valid/invalid/concurrent calls مع نتائج ثابتة | int80 وVFS/IPC/Exit مثبت؛ ABI versioning وwait لاحقان |
| Capabilities/IPC | transferable capabilities، ports، blocking/wakeup، revocation | no ambient handles، stale-token rejection، cross-process transfer policy | opaque VFS/IPC handles وrevoke مثبتة؛ transfer/blocking لاحق |
| Storage | block abstraction، cache، partition، persistent FS، recovery | create/read/write/fsck، power-loss/recovery test | ATA PIO وMBR وNovelFS fixed-layout RW probe؛ FS عام لاحق |
| Devices | PCI، virtio، ATA/AHCI، keyboard/display، DMA/IOMMU | enumeration، ownership، timeout/error recovery، isolation | ATA PIO probe وPS/2 boundary؛ PCI/DMA/AHCI لاحق |
| Network | Ethernet، ARP، IPv4/IPv6 policy، UDP/TCP sockets | loopback ثم virtio/NIC roundtrip مع ownership/backpressure | bounded loopback/datagram فقط |
| Userland | libc/runtime، init supervisor، shell، coreutils، services | boot إلى shell، process launch، redirection، signals، logs | ELF syscall fixtures فقط |
| Distribution | package format، repository، installer، config، upgrade/recovery | install/upgrade/rollback من image موثقة | غير منجز |
| Security | threat model، least privilege، fuzzing، hardening، signing | negative suite وaudit reports وsigned artifacts | capability smoke وfault isolation فقط |
| Release | CI، reproducible builds، BIOS/UEFI artifacts، support matrix | clean checkout، hashes، QEMU matrix، release notes | local clean build/gate للـBIOS فقط |

## ترتيب العمل

يُنفذ العمل بهذا الترتيب: platform/memory contracts، ثم dynamic lifecycle/scheduler، ثم syscall and capability ABI، ثم persistent filesystem، ثم device/network paths، ثم userland supervisor/shell، ثم packaging/installer، ثم security and release qualification. لا تُربط طبقة أعلى بعقد سفلي تجريبي دون إبقاء حدودها معلنة.

## تعريف التوزيعة الحقيقية

لا يستخدم وصف **توزيعة حقيقية** إلا بعد أن يقلع النظام إلى init supervisor، ويستطيع تشغيل أكثر من binary من filesystem دائم، ويمتلك shell وأدوات أساسية، ويقرأ ويكتب تخزينًا قابلًا للاستعادة، ويطبق capability policy على processes/devices، ويملك installer وupgrade/recovery، وتنجح صور الإصدار من clean build وCI وQEMU matrix. قبل ذلك يبقى الوصف الصحيح هو prototype أو preview بحسب الدليل.
