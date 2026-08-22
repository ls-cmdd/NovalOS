# Novel OS — Release Roadmap

هذه الخطة تفصل بين **الكود العامل** و**الاختبار التمهيدي** و**الجاهزية للإطلاق**. لا تُغلق أي مهمة إلا إذا كان لها implementation فعلي، اختبار قابل لإعادة التشغيل، وسجل QEMU أو artifact يثبت النتيجة.

| الترتيب | المهمة | الحالة الحالية | ما يلزم قبل الإطلاق العام |
|---:|---|---|---|
| 1 | build reproducibility | جزئي منجز — workspace وCargo.lock وnightly وbootloader vendor patch وBIOS build متسلسل | clean-clone validation في بيئة مستقلة، CI خارجي، وتثبيت toolchain hash |
| 2 | release gate وfault scan | منجز للـBIOS prototype — fmt/release build/QEMU markers وفحص panic/triple fault | matrix لعدة RAM/CPU/QEMU versions وnegative tests |
| 3 | memory correctness | جزئي — frame allocator وoffset paging وisolated CR3 roots وmap/teardown smoke | ownership ledger، unmap/recycle فعلي لكل leaf، heap guard، وASLR أو layout policy موثقة |
| 4 | hardware interrupt baseline | جزئي منجز — GDT/IDT/PIC/PIT/keyboard وEOI مثبتة | APIC/IOAPIC routing، SMP-safe locking، وinterrupt storm tests |
| 5 | user page tables وRing3 | منجز prototype — user code/stack/guard وU/S flags وiretq وCPL3 int80 مثبتة | formal page-table permission tests وkernel mapping audit |
| 6 | process lifecycle | جزئي — TCB states، Exit/Fault، revoke، root reclaim، saved-context handoff وidle hlt مثبتة | zombie/reparent policy، wait/join، PID reuse generation، per-process resource accounting |
| 7 | scheduler | جزئي — bounded capacity=3، PIT GP/frame save، Ready/Running، per-process kernel stacks، saved resume | dynamic run queue، idle task حقيقي، priorities/fairness، FPU/SIMD/XSAVE، preemption-safe locks |
| 8 | syscall ABI | جزئي — int80 write/open/read/file_write/close/exit مع pointer validation | versioned ABI، full errno contract، syscall table capability checks، copy-in/copy-out بدون raw shared references، وsyscall/sysret evaluation |
| 9 | capability security | جزئي منجز — VFS namespace/slot generation tokens وIPC port generation handles، rights/owner checks، revoke/reopen وrevoke_ports | userspace delegation/transfer، device capabilities، namespace lifetime audit، confused-deputy وconcurrent access tests |
| 10 | ELF/init userspace | جزئي — deterministic init/exit ELF، PT_LOAD/BSS/overflow checks، initramfs syscalls | relocations/interpreter policy، multiple binaries، shell/init supervisor، signals، env/args، ABI libc أو runtime |
| 11 | filesystem | جزئي — in-memory VFS وramdisk reader وopaque handles؛ NovelFS fixed-layout يقرأ ويكتب scratch sector مع rollback | block cache، mount interface عام، directory mutation، journaling/recovery، fsck، concurrent writers |
| 12 | block devices | جزئي منجز — ATA PIO read/write وMBR parsing وQEMU sector verification | IDENTIFY، IRQ/DMA path، robust recovery، partition mutation policy، hot/error behavior |
| 13 | IPC/network | جزئي — userspace port_create/send/recv وbounded queue وowner/generation revoke، إضافة إلى loopback packets/datagrams/checksum/backpressure | blocking/wakeup، cross-process delegation، UDP sockets، Ethernet/ARP، NIC driver، DMA/IOMMU policy |
| 14 | security/fault suite | جزئي — guard fault وinvalid pointer/ELF وstale capabilities smoke | fuzzing، stack/heap overflow، malformed packets، syscall concurrency، KASLR/SMEP/SMAP policy حيث يدعم platform |
| 15 | platform | BIOS فقط | UEFI، ACPI، APIC/IOAPIC، SMP، timers، framebuffer portability، secure boot/signing |
| 16 | userland/distribution | غير منجز كـdistribution — ELF fixtures وVFS/IPC syscall clients فقط، بلا shell أو services | base utilities، shell، init supervisor، package format/repository، installer، config، docs، upgrade/recovery |
| 17 | release packaging | prototype منجز — BIOS artifact/hash/manifest وrelease gate | UEFI/SMP artifacts، signed images، installer، CI artifacts، reproducible clean-clone report |

## ترتيب التنفيذ الإلزامي

يُستكمل أولًا process lifecycle وdynamic scheduler وsyscall/capability ABI، ثم persistent filesystem فوق block boundary، ثم network/device ownership، ثم userland base. لا يُستخدم وصف **توزيعة مكتملة** أو **release candidate** قبل وجود userland supervisor، persistent storage، device/network policy، UEFI/SMP policy، وCI خارجي يكرر الأدلة.

## الحالة الصادقة الحالية

Novel OS الآن **BIOS-only development prototype قابل للإقلاع والاختبار**. آخر artifact يثبت kernel وRing3 وVFS/capabilities وpreemption وExit/Fault/reap وATA sector probe، لكنه لا يحقق بعد نظام تشغيل عام قويًا أو توزيعة مكتملة.
