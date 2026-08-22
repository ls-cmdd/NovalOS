# Novel OS — Verification Report

## النطاق المثبت

هذه النسخة هي prototype kernel قابل للإقلاع، لا نظام تشغيل مكتمل متعدد المستخدمين. تم اختبار كل طبقة بعد دمجها مع الطبقات السابقة، وتُحفظ القيود غير المنجزة في `docs/backlog.md` و`docs/architecture.md` بدل تقديمها كميزات مكتملة.

## الاختبارات

| الاختبار | النتيجة | الدليل |
|---|---|---|
| `cargo fmt --all -- --check` | ناجح | لا توجد فروقات تنسيق |
| `cargo build --release --locked` | ناجح | نواة `x86_64-unknown-none` وصورة BIOS release من workspace الحالي |
| clean checkout package | ناجح جزئيًا | clean vendored checkout السابق من commit `5b5862a` مرّ release/QEMU/hash proof؛ أحدث package من tag `user-preemption-v1` مرّ QEMU gate وSHA256، BIOS hash=`b0bd612310933dd95d22190d05ea0a82990bdb51d652fef21998d4d5c1ef45b1` وCargo.lock hash=`d9e069db82aa7baf2400e6130028d62e9efe65c1170cc4bcffedd5a51a55fb66`؛ لم يُعد clean-clone hash comparison بعد آخر user-preemption commit |
| GDT/data/TSS/IST | ناجح | تحميل selectors وTSS ومكدس double fault |
| IDT CPU exceptions | ناجح | `int3` يعود إلى kernel دون triple fault |
| PIC remap | ناجح | IRQ0 يصل إلى vector 0x20 بدل 0x08 |
| APIC detection | ناجح جزئيًا | CPUID: xAPIC=false وx2APIC=false في QEMU الحالي؛ PIC يبقى controller الفعلي |
| frame allocator | ناجح | أكثر من 31 ألف usable 4KiB frame في QEMU |
| frame recycle | ناجح | `frame recycle invariant: true` |
| heap paging | ناجح | 25 صفحة عند heap virtual range |
| heap translation | ناجح | `heap translation: true` |
| user code/stack/guard mapping | ناجح جزئيًا | code عند `0x400000`، stack عند `0x7ff000`، guard غير مترجمة عند `0x7fe000`، وflags مقروءة مباشرة من page-table entries |
| process address-space root | ناجح جزئيًا | rootان PML4 مستقلان، kernel mappings منسوخة، user PML4 slot مستقل، وإطارا code مختلفان بعد CR3 switch/restore؛ ownership الكامل متعدد العمليات لاحق |
| GlobalAlloc | ناجح | Vec capacity 64 وchecksum 53,982 في آخر QEMU run |
| PIT | ناجح | 100Hz تقريبًا، و4 ticks في smoke test، مع تسليم IRQ0 للـpreemptive scheduler |
| PS/2 queue | ناجح | IRQ1 handler وfixed scancode queue؛ لا scancode pending في التشغيل الآلي |
| scheduler model | ناجح أوليًا | النموذج التعاوني القديم مثبت بمهمتين انتهتا، queue فارغ، و2 runs؛ يوجد أيضًا proof منفصل لمهمتي kernel preemptive |
| context switch | ناجح أوليًا | انتقال إلى stack مستقل والعودة: `context switch smoke: true`؛ وtimer-driven alternation لمهمتي kernel stack مستقلتين مثبتة في QEMU |
| VFS | ناجح جزئيًا | create/read/list/unlink مع opaque handles، read/write rights، offset، وإغلاق ورفض stale handles |
| filesystem handle boundary | ناجح جزئيًا | `open/read/file_write/close` نفذت من init CPL3 عبر int 0x80، مع user-buffer validation؛ VFS يستخدم CapabilityNamespace صريحًا وopaque token يحوي namespace/slot generations، ويثبت wrong-owner وstale-token وrevoke/reopen rejection؛ reaper revokes namespace لكل owner |
| ramdisk | ناجح أوليًا | embedded `ramdisk/init.txt` قُرئ عبر VFS وinit.elf deterministic بحجم 448 bytes في loader report |
| loopback protocol | ناجح جزئيًا | bounded datagram round-trip بالمنافذ 1000→2000، checksum validation وcorruption rejection؛ لا NIC أو UDP خارجي |
| syscall ABI | ناجح جزئيًا | int 0x80 wrapper يمرر `rax/rdi/rsi/rdx`، يتحقق من pointer وlength، وينفذ write/open/read/file_write/close ثم yield؛ Exit يعيد توجيه iret frame إلى kernel reaper |
| process teardown | ناجح جزئيًا | process reaper يستعيد kernel CR3 ويفك code/stack ويعيد root والـtracked intermediate frames؛ rollback لمسار MapToError صار صريحًا، وTCB يسجل owner/state/status، وQEMU أثبت bounded queue لثلاثة roots مع حذف frame الخارج وreclaim؛ generalized queue وidle policy لاحقان |
| user fault policy | ناجح جزئيًا | guard-page `#PF` من CPL3 يسجل السبب، يقتل العملية، يحول TCB إلى `Faulted`، ويحوّلها إلى reaper دون تعديل RIP؛ ثلاثة roots user تعرضت للسياسة بعد PIT frame switching، مع revocation/reclaim لكل owner |
| ELF PT_LOAD loader | ناجح للـfixture | loader نسخ PT_LOAD إلى user mapping وملأ BSS بالصفر تحت root مستقل، ثم نجح teardown |
| init userspace from ramdisk | ناجح جزئيًا | `init.elf` قُرئ من VFS، حُمّل إلى ثلاثة roots مستقلة، ودخلت contexts عبر `iretq` إلى CPL3؛ PIT يبدّل بينها ويحذف context الخارج ثم يعيد CR3، مع revocation/reclaim لكل owner؛ `bytes=448` هو PT_LOAD bytes، بينما ملف ELF الكلي 568 bytes |
| Ring 3 transition | ناجح | `iretq` إلى `0x400000` مع user stack `0x7ff000`، ثم `RING3 int80: cpl=3 valid=true` في QEMU |

## QEMU evidence

```text
NOVEL_OS :: PHASE 3 MEMORY MANAGEMENT
GDT + TSS + IDT + PIC loaded; APIC xAPIC=false x2APIC=false id=0
frame recycle invariant: true
heap translation: true
user mappings: code=0x400000 stack=0x7ff000 guard=0x7fe000 valid=true
user mapping permissions: true
process address space CR3 isolation: true
process teardown: first=true second=true
ELF loader PT_LOAD+BSS: true
init ELF from ramdisk: fs=true loaded=true bytes=448
process table: init registered=true current=true state=Running worker=2 state=Ready worker=3 state=Ready
user preemption: contexts=3 timer_frame=true
paging + heap ready: Vec capacity=64, checksum=53982
scheduler smoke: completed=2, pending=0, runs=2
context switch smoke: true
preemptive scheduler: task0=4 task1=4 switches=9 starvation=false
VFS smoke: true, handles: true, ramdisk: true, global=true
capability table: namespace=true opaque=true generation=true revoked=true
loopback: smoke=true checksum=true backpressure=true
loopback datagram: roundtrip=true ports=1000->2000 checksum=true corruption_rejected=true
syscall ABI smoke: true
hardware IRQs online: 4 timer ticks
keyboard queue ready: no pending scancode
breakpoint exception works after hardware IRQs
RING3 transition: init ELF iretq submitted
RING3 int80: cpl=3 valid=true
init from elf
SYSCALL write: accepted length=14
RING3 int80: cpl=3 valid=true
SYSCALL open: handle=1
RING3 int80: cpl=3 valid=true
SYSCALL read: handle=1 bytes=8
RING3 int80: cpl=3 valid=true
SYSCALL file_write: handle=1 bytes=8
RING3 int80: cpl=3 valid=true
NOVEL_OS
SYSCALL write: accepted length=8
RING3 int80: cpl=3 valid=true
SYSCALL close: handle=1 closed=true
RING3 int80: cpl=3 valid=true
SYSCALL dispatch: number=2 return=0
RING3 int80: cpl=3 valid=true
SYSCALL write: rejected invalid_user_pointer
USER FAULT POLICY: kill=true status=61444 address=Ok(VirtAddr(0x7fe000)) error=PageFaultErrorCode(USER_MODE)
user preemption: switches=15 frames=true starvation=false
PROCESS EXIT: status=61444 owner=1 state=Faulted revoked_handles=1 reclaimed=true next=true
PROCESS HANDOFF: from=1 to=2 state=Running cr3=switched active=true
user preemption: switches=16 frames=true starvation=false
PROCESS EXIT: status=61444 owner=3 state=Faulted revoked_handles=1 reclaimed=true next=true
PROCESS HANDOFF: from=3 to=2 state=Running cr3=switched active=true
user preemption: switches=16 frames=true starvation=false
PROCESS EXIT: status=61444 owner=2 state=Faulted revoked_handles=3 reclaimed=true next=false
USER QUEUE: idle=true runnable=0
```

تنتهي جلسة الاختبار بمهلة لأن النواة تدخل حلقة `hlt` بعد اكتمال التهيئة. لذلك فإن exit status `124` من harness متوقع ولا يدل على فشل البناء أو الإقلاع. ترتيب المالكين وعدد preemption switches يتغيران باختلاف توقيت QEMU؛ gate يتحقق من markers الدلالية ومن وجود ثلاثة exits وidle، لا من ترتيب أو عدد ثابت.

## ما لم يُعلن منجزًا

لم يُنفّذ بعد `syscall/sysret`، أو dynamic run queue التي تدير عددًا غير محدود من العمليات؛ المنجز الحالي هو bounded capacity=3 مع اختيار runnable slots، حذف contexts الخارجة، وidle `hlt` بعد آخر reclaim. CapabilityNamespace وopaque generation tokens منفذان ومثبتان داخل VFS، لكن multi-process handle policy الشاملة خارج VFS وdelegation/transfer semantics لم تُثبت بعد. المنجز الحالي يثبت حفظ/استعادة user interrupt frame لثلاثة roots، CR3 handoff، exit/fault teardown وowner revocation، وloopback datagram bounded فقط. كما لم يُنفّذ APIC/IOAPIC routing أو persistent filesystem أو NIC/DMA driver أو UDP خارجي.
