# Final verification evidence

commit=56c57da31509e32a12813818be47885dcc92bd97
tag=smp-preempt-state-lock-v1

## package-release
[1m[33mwarning[0m[1m: unnecessary `unsafe` block[0m
  [1m[94m--> [0mkernel/src/apic.rs:54:20
   [1m[94m|[0m
[1m[94m54[0m [1m[94m|[0m         let leaf = unsafe { core::arch::x86_64::__cpuid(1) };
   [1m[94m|[0m                    [1m[33m^^^^^^[0m [1m[33munnecessary `unsafe` block[0m
   [1m[94m|[0m
   [1m[94m= [0m[1mnote[0m: `#[warn(unused_unsafe)]` (part of `#[warn(unused)]`) on by default

[1m[33mwarning[E0133][0m[1m: use of mutable static is unsafe and requires unsafe block[0m
  [1m[94m--> [0mkernel/src/heap.rs:18:41
   [1m[94m|[0m
[1m[94m18[0m [1m[94m|[0m     let start = core::ptr::addr_of_mut!(HEAP_STORAGE.0) as *mut u8;
   [1m[94m|[0m                                         [1m[33m^^^^^^^^^^^^[0m [1m[33muse of mutable static[0m
   [1m[94m|[0m
   [1m[94m= [0m[1mnote[0m: mutable statics can be mutated by multiple threads: aliasing violations or data races will cause undefined behavior
[1m[92mnote[0m: an unsafe function restricts its caller, but its body is safe by default
  [1m[94m--> [0mkernel/src/heap.rs:17:1
   [1m[94m|[0m
[1m[94m17[0m [1m[94m|[0m pub unsafe fn init() {
   [1m[94m|[0m [1m[92m^^^^^^^^^^^^^^^^^^^^[0m
   [1m[94m= [0m[1mnote[0m: for more information, see <https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-op-in-unsafe-fn.html>
   [1m[94m= [0m[1mnote[0m: `#[warn(unsafe_op_in_unsafe_fn)]` (part of `#[warn(rust_2024_compatibility)]`) on by default

[1m[33mwarning[E0133][0m[1m: call to unsafe function `memory::active_level_4_table` is unsafe and requires unsafe block[0m
   [1m[94m--> [0mkernel/src/memory.rs:131:25
    [1m[94m|[0m
[1m[94m131[0m [1m[94m|[0m     let level_4_table = active_level_4_table(physical_memory_offset);
    [1m[94m|[0m                         [1m[33m^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^[0m [1m[33mcall to unsafe function[0m
    [1m[94m|[0m
    [1m[94m= [0m[1mnote[0m: consult the function's documentation for information on how to avoid undefined behavior
[1m[92mnote[0m: an unsafe function restricts its caller, but its body is safe by default
   [1m[94m--> [0mkernel/src/memory.rs:130:1
    [1m[94m|[0m
[1m[94m130[0m [1m[94m|[0m pub unsafe fn init_offset_page_table(physical_memory_offset: VirtAddr) -> OffsetPageTable<'static> {
    [1m[94m|[0m [1m[92m^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^[0m
    [1m[94m= [0m[1mnote[0m: for more information, see <https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-op-in-unsafe-fn.html>

[1m[33mwarning[E0133][0m[1m: call to unsafe function `x86_64::structures::paging::OffsetPageTable::<'a>::new` is unsafe and requires unsafe block[0m
   [1m[94m--> [0mkernel/src/memory.rs:132:5
    [1m[94m|[0m
[1m[94m132[0m [1m[94m|[0m     OffsetPageTable::new(level_4_table, physical_memory_offset)
    [1m[94m|[0m     [1m[33m^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^[0m [1m[33mcall to unsafe function[0m
    [1m[94m|[0m
    [1m[94m= [0m[1mnote[0m: consult the function's documentation for information on how to avoid undefined behavior
    [1m[94m= [0m[1mnote[0m: for more information, see <https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-op-in-unsafe-fn.html>

[1m[33mwarning[E0133][0m[1m: call to unsafe function `x86_64::structures::paging::OffsetPageTable::<'a>::new` is unsafe and requires unsafe block[0m
   [1m[94m--> [0mkernel/src/memory.rs:200:22
    [1m[94m|[0m
[1m[94m200[0m [1m[94m|[0m     let mut mapper = OffsetPageTable::new(root_table, offset);
    [1m[94m|[0m                      [1m[33m^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^[0m [1m[33mcall to unsafe function[0m
    [1m[94m|[0m
    [1m[94m= [0m[1mnote[0m: consult the function's documentation for information on how to avoid undefined behavior
[1m[92mnote[0m: an unsafe function restricts its caller, but its body is safe by default
   [1m[94m--> [0mkernel/src/memory.rs:191:1
    [1m[94m|[0m
[1m[94m191[0m [1m[94m|[0m [1m[92m/[0m unsafe fn map_user_pages(
[1m[94m192[0m [1m[94m|[0m [1m[92m|[0m     root: PhysFrame<Size4KiB>,
[1m[94m193[0m [1m[94m|[0m [1m[92m|[0m     code: PhysFrame<Size4KiB>,
[1m[94m194[0m [1m[94m|[0m [1m[92m|[0m     stack: PhysFrame<Size4KiB>,
[1m[94m195[0m [1m[94m|[0m [1m[92m|[0m     offset: VirtAddr,
[1m[94m196[0m [1m[94m|[0m [1m[92m|[0m     allocator: &mut BootInfoFrameAllocator,
[1m[94m197[0m [1m[94m|[0m [1m[92m|[0m ) -> bool {
    [1m[94m|[0m [1m[92m|_________^[0m
    [1m[94m= [0m[1mnote[0m: for more information, see <https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-op-in-unsafe-fn.html>

[1m[33mwarning[E0133][0m[1m: call to unsafe function `x86_64::structures::paging::Mapper::map_to` is unsafe and requires unsafe block[0m
   [1m[94m--> [0mkernel/src/memory.rs:204:20
    [1m[94m|[0m
[1m[94m204[0m [1m[94m|[0m     let code_map = mapper.map_to(code_page, code, flags, allocator);
    [1m[94m|[0m                    [1m[33m^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^[0m [1m[33mcall to unsafe function[0m
    [1m[94m|[0m
    [1m[94m= [0m[1mnote[0m: consult the function's documentation for information on how to avoid undefined behavior
    [1m[94m= [0m[1mnote[0m: for more information, see <https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-op-in-unsafe-fn.html>

[1m[33mwarning[E0133][0m[1m: call to unsafe function `x86_64::structures::paging::Mapper::map_to` is unsafe and requires unsafe block[0m
   [1m[94m--> [0mkernel/src/memory.rs:205:21
    [1m[94m|[0m
[1m[94m205[0m [1m[94m|[0m       let stack_map = mapper.map_to(
    [1m[94m|[0m [1m[33m _____________________^[0m
[1m[94m206[0m [1m[94m|[0m [1m[33m|[0m         stack_page,
[1m[94m207[0m [1m[94m|[0m [1m[33m|[0m         stack,
[1m[94m208[0m [1m[94m|[0m [1m[33m|[0m         flags | PageTableFlags::WRITABLE,
[1m[94m209[0m [1m[94m|[0m [1m[33m|[0m         allocator,
[1m[94m210[0m [1m[94m|[0m [1m[33m|[0m     );
    [1m[94m|[0m [1m[33m|_____^[0m [1m[33mcall to unsafe function[0m
    [1m[94m|[0m
    [1m[94m= [0m[1mnote[0m: consult the function's documentation for information on how to avoid undefined behavior
    [1m[94m= [0m[1mnote[0m: for more information, see <https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-op-in-unsafe-fn.html>

[1m[33mwarning[0m[1m: variants `MissingRsdp` and `ShortTable` are never constructed[0m
  [1m[94m--> [0mkernel/src/acpi.rs:13:5
   [1m[94m|[0m
[1m[94m12[0m [1m[94m|[0m pub enum AcpiError {
   [1m[94m|[0m          [1m[94m---------[0m [1m[94mvariants in this enum[0m
[1m[94m13[0m [1m[94m|[0m     MissingRsdp,
   [1m[94m|[0m     [1m[33m^^^^^^^^^^^[0m
[1m[94m...[0m
[1m[94m16[0m [1m[94m|[0m     ShortTable,
   [1m[94m|[0m     [1m[33m^^^^^^^^^^[0m
   [1m[94m|[0m
   [1m[94m= [0m[1mnote[0m: `AcpiError` has derived impls for the traits `Clone` and `Debug`, but these are intentionally ignored during dead code analysis
   [1m[94m= [0m[1mnote[0m: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default

[1m[33mwarning[0m[1m: constant `LAPIC_ICR_LOW` is never used[0m
  [1m[94m--> [0mkernel/src/apic.rs:17:7
   [1m[94m|[0m
[1m[94m17[0m [1m[94m|[0m const LAPIC_ICR_LOW: u64 = 0x300;
   [1m[94m|[0m       [1m[33m^^^^^^^^^^^^^[0m

[1m[33mwarning[0m[1m: constant `LAPIC_ICR_HIGH` is never used[0m
  [1m[94m--> [0mkernel/src/apic.rs:18:7
   [1m[94m|[0m
[1m[94m18[0m [1m[94m|[0m const LAPIC_ICR_HIGH: u64 = 0x310;
   [1m[94m|[0m       [1m[33m^^^^^^^^^^^^^^[0m

[1m[33mwarning[0m[1m: function `start_ap` is never used[0m
   [1m[94m--> [0mkernel/src/apic.rs:132:8
    [1m[94m|[0m
[1m[94m132[0m [1m[94m|[0m pub fn start_ap(local_id: u32, startup_vector: u8) -> bool {
    [1m[94m|[0m        [1m[33m^^^^^^^^[0m

[1m[33mwarning[0m[1m: function `wait_for_icr_idle` is never used[0m
   [1m[94m--> [0mkernel/src/apic.rs:159:4
    [1m[94m|[0m
[1m[94m159[0m [1m[94m|[0m fn wait_for_icr_idle() -> bool {
    [1m[94m|[0m    [1m[33m^^^^^^^^^^^^^^^^^[0m

[1m[33mwarning[0m[1m: function `build_idt_at` is never used[0m
  [1m[94m--> [0mkernel/src/interrupts.rs:85:15
   [1m[94m|[0m
[1m[94m85[0m [1m[94m|[0m pub unsafe fn build_idt_at(address: VirtAddr) -> bool {
   [1m[94m|[0m               [1m[33m^^^^^^^^^^^^[0m

[1m[33mwarning[0m[1m: constant `SLOT_BITS` is never used[0m
 [1m[94m--> [0mkernel/src/ipc.rs:6:7
  [1m[94m|[0m
[1m[94m6[0m [1m[94m|[0m const SLOT_BITS: u32 = 3;
  [1m[94m|[0m       [1m[33m^^^^^^^^^[0m

[1m[33mwarning[0m[1m: method `allocate_frame_below` is never used[0m
  [1m[94m--> [0mkernel/src/memory.rs:65:12
   [1m[94m|[0m
[1m[94m23[0m [1m[94m|[0m impl BootInfoFrameAllocator {
   [1m[94m|[0m [1m[94m---------------------------[0m [1m[94mmethod in this implementation[0m
[1m[94m...[0m
[1m[94m65[0m [1m[94m|[0m     pub fn allocate_frame_below(&mut self, limit: u64) -> Option<PhysFra[1m[94m...[0m
   [1m[94m|[0m            [1m[33m^^^^^^^^^^^^^^^^^^^^[0m

[1m[33mwarning[0m[1m: constant `TRAMPOLINE_LIMIT` is never used[0m
  [1m[94m--> [0mkernel/src/smp.rs:13:7
   [1m[94m|[0m
[1m[94m13[0m [1m[94m|[0m const TRAMPOLINE_LIMIT: u64 = 0x10_0000;
   [1m[94m|[0m       [1m[33m^^^^^^^^^^^^^^^^[0m

[1m[33mwarning[0m[1m: constant `PML4_SLOT` is never used[0m
  [1m[94m--> [0mkernel/src/smp.rs:14:7
   [1m[94m|[0m
[1m[94m14[0m [1m[94m|[0m const PML4_SLOT: usize = 16;
   [1m[94m|[0m       [1m[33m^^^^^^^^^[0m

[1m[33mwarning[0m[1m: constant `STACK_SLOT` is never used[0m
  [1m[94m--> [0mkernel/src/smp.rs:15:7
   [1m[94m|[0m
[1m[94m15[0m [1m[94m|[0m const STACK_SLOT: usize = 24;
   [1m[94m|[0m       [1m[33m^^^^^^^^^^[0m

[1m[33mwarning[0m[1m: constant `CPU_SLOT` is never used[0m
  [1m[94m--> [0mkernel/src/smp.rs:16:7
   [1m[94m|[0m
[1m[94m16[0m [1m[94m|[0m const CPU_SLOT: usize = 32;
   [1m[94m|[0m       [1m[33m^^^^^^^^[0m

[1m[33mwarning[0m[1m: constant `ENTRY_SLOT` is never used[0m
  [1m[94m--> [0mkernel/src/smp.rs:17:7
   [1m[94m|[0m
[1m[94m17[0m [1m[94m|[0m const ENTRY_SLOT: usize = 40;
   [1m[94m|[0m       [1m[33m^^^^^^^^^^[0m

[1m[33mwarning[0m[1m: constant `LONG_ENTRY_SLOT` is never used[0m
  [1m[94m--> [0mkernel/src/smp.rs:18:7
   [1m[94m|[0m
[1m[94m18[0m [1m[94m|[0m const LONG_ENTRY_SLOT: usize = 48;
   [1m[94m|[0m       [1m[33m^^^^^^^^^^^^^^^[0m

[1m[33mwarning[0m[1m: constant `ONLINE_SLOT` is never used[0m
  [1m[94m--> [0mkernel/src/smp.rs:19:7
   [1m[94m|[0m
[1m[94m19[0m [1m[94m|[0m const ONLINE_SLOT: usize = 56;
   [1m[94m|[0m       [1m[33m^^^^^^^^^^^[0m

[1m[33mwarning[0m[1m: constant `IDT_SLOT` is never used[0m
  [1m[94m--> [0mkernel/src/smp.rs:20:7
   [1m[94m|[0m
[1m[94m20[0m [1m[94m|[0m const IDT_SLOT: usize = 64;
   [1m[94m|[0m       [1m[33m^^^^^^^^[0m

[1m[33mwarning[0m[1m: constant `STAGE_SLOT` is never used[0m
  [1m[94m--> [0mkernel/src/smp.rs:21:7
   [1m[94m|[0m
[1m[94m21[0m [1m[94m|[0m const STAGE_SLOT: usize = 72;
   [1m[94m|[0m       [1m[33m^^^^^^^^^^[0m

[1m[33mwarning[0m[1m: constant `PER_CPU_SLOT` is never used[0m
  [1m[94m--> [0mkernel/src/smp.rs:22:7
   [1m[94m|[0m
[1m[94m22[0m [1m[94m|[0m const PER_CPU_SLOT: usize = 80;
   [1m[94m|[0m       [1m[33m^^^^^^^^^^^^[0m

[1m[33mwarning[0m[1m: constant `LONG_MODE_OFFSET` is never used[0m
  [1m[94m--> [0mkernel/src/smp.rs:23:7
   [1m[94m|[0m
[1m[94m23[0m [1m[94m|[0m const LONG_MODE_OFFSET: u64 = 0x180;
   [1m[94m|[0m       [1m[33m^^^^^^^^^^^^^^^^[0m

[1m[33mwarning[0m[1m: constant `PER_CPU_OFFLINE` is never used[0m
  [1m[94m--> [0mkernel/src/smp.rs:25:7
   [1m[94m|[0m
[1m[94m25[0m [1m[94m|[0m const PER_CPU_OFFLINE: u8 = 0;
   [1m[94m|[0m       [1m[33m^^^^^^^^^^^^^^^[0m

[1m[33mwarning[0m[1m: constant `PER_CPU_LONG_MODE` is never used[0m
  [1m[94m--> [0mkernel/src/smp.rs:26:7
   [1m[94m|[0m
[1m[94m26[0m [1m[94m|[0m const PER_CPU_LONG_MODE: u8 = 1;
   [1m[94m|[0m       [1m[33m^^^^^^^^^^^^^^^^^[0m

[1m[33mwarning[0m[1m: associated items `new` and `reset` are never used[0m
  [1m[94m--> [0mkernel/src/smp.rs:46:14
   [1m[94m|[0m
[1m[94m45[0m [1m[94m|[0m impl PerCpuState {
   [1m[94m|[0m [1m[94m----------------[0m [1m[94massociated items in this implementation[0m
[1m[94m46[0m [1m[94m|[0m     const fn new() -> Self {
   [1m[94m|[0m              [1m[33m^^^[0m
[1m[94m...[0m
[1m[94m61[0m [1m[94m|[0m     fn reset(&self, self_base: u64, apic_id: u64, stack_top: u64) {
   [1m[94m|[0m        [1m[33m^^^^^[0m

[1m[33mwarning[0m[1m: struct `PerCpuSnapshot` is never constructed[0m
  [1m[94m--> [0mkernel/src/smp.rs:89:12
   [1m[94m|[0m
[1m[94m89[0m [1m[94m|[0m pub struct PerCpuSnapshot {
   [1m[94m|[0m            [1m[33m^^^^^^^^^^^^^^[0m

[1m[33mwarning[0m[1m: function `per_cpu_snapshot` is never used[0m
   [1m[94m--> [0mkernel/src/smp.rs:103:15
    [1m[94m|[0m
[1m[94m103[0m [1m[94m|[0m pub unsafe fn per_cpu_snapshot(base: u64, index: usize) -> Option<PerCp[1m[94m...[0m
    [1m[94m|[0m               [1m[33m^^^^^^^^^^^^^^^^[0m

[1m[33mwarning[0m[1m: function `initialize_per_cpu_frame` is never used[0m
   [1m[94m--> [0mkernel/src/smp.rs:128:11
    [1m[94m|[0m
[1m[94m128[0m [1m[94m|[0m unsafe fn initialize_per_cpu_frame(base: u64) {
    [1m[94m|[0m           [1m[33m^^^^^^^^^^^^^^^^^^^^^^^^[0m

[1m[33mwarning[0m[1m: function `count_per_cpu_online` is never used[0m
   [1m[94m--> [0mkernel/src/smp.rs:196:4
    [1m[94m|[0m
[1m[94m196[0m [1m[94m|[0m fn count_per_cpu_online(base: u64, limit: usize) -> u32 {
    [1m[94m|[0m    [1m[33m^^^^^^^^^^^^^^^^^^^^[0m

[1m[33mwarning[0m[1m: function `bring_up` is never used[0m
   [1m[94m--> [0mkernel/src/smp.rs:236:8
    [1m[94m|[0m
[1m[94m236[0m [1m[94m|[0m pub fn bring_up(
    [1m[94m|[0m        [1m[33m^^^^^^^^[0m

[1m[33mwarning[0m[1m: function `ensure_kernel_coverage` is never used[0m
   [1m[94m--> [0mkernel/src/smp.rs:548:4
    [1m[94m|[0m
[1m[94m548[0m [1m[94m|[0m fn ensure_kernel_coverage(
    [1m[94m|[0m    [1m[33m^^^^^^^^^^^^^^^^^^^^^^[0m

[1m[33mwarning[0m[1m: function `install_trampoline` is never used[0m
   [1m[94m--> [0mkernel/src/smp.rs:593:4
    [1m[94m|[0m
[1m[94m593[0m [1m[94m|[0m fn install_trampoline(physical: u64, offset: VirtAddr) -> bool {
    [1m[94m|[0m    [1m[33m^^^^^^^^^^^^^^^^^^[0m

[1m[33mwarning[0m[1m: function `patch_trampoline` is never used[0m
   [1m[94m--> [0mkernel/src/smp.rs:609:4
    [1m[94m|[0m
[1m[94m609[0m [1m[94m|[0m fn patch_trampoline(
    [1m[94m|[0m    [1m[33m^^^^^^^^^^^^^^^^[0m

[1m[33mwarning[0m[1m: function `wait_online` is never used[0m
   [1m[94m--> [0mkernel/src/smp.rs:641:4
    [1m[94m|[0m
[1m[94m641[0m [1m[94m|[0m fn wait_online(offset: VirtAddr, online_physical: u64, _index: usize) -[1m[94m...[0m
    [1m[94m|[0m    [1m[33m^^^^^^^^^^^[0m

[1m[33mwarning[0m[1m: variant `Exited` is never constructed[0m
  [1m[94m--> [0mkernel/src/syscall.rs:28:5
   [1m[94m|[0m
[1m[94m26[0m [1m[94m|[0m pub enum Return {
   [1m[94m|[0m          [1m[94m------[0m [1m[94mvariant in this enum[0m
[1m[94m27[0m [1m[94m|[0m     Value(u64),
[1m[94m28[0m [1m[94m|[0m     Exited(u64),
   [1m[94m|[0m     [1m[33m^^^^^^[0m
   [1m[94m|[0m
   [1m[94m= [0m[1mnote[0m: `Return` has derived impls for the traits `Debug` and `Clone`, but these are intentionally ignored during dead code analysis

[1m[33mwarning[0m[1m: fields `code` and `stack` are never read[0m
  [1m[94m--> [0mkernel/src/user.rs:15:9
   [1m[94m|[0m
[1m[94m14[0m [1m[94m|[0m pub struct UserMapping {
   [1m[94m|[0m            [1m[94m-----------[0m [1m[94mfields in this struct[0m
[1m[94m15[0m [1m[94m|[0m     pub code: PhysFrame<Size4KiB>,
   [1m[94m|[0m         [1m[33m^^^^[0m
[1m[94m16[0m [1m[94m|[0m     pub stack: PhysFrame<Size4KiB>,
   [1m[94m|[0m         [1m[33m^^^^^[0m
   [1m[94m|[0m
   [1m[94m= [0m[1mnote[0m: `UserMapping` has a derived impl for the trait `Clone`, but this is intentionally ignored during dead code analysis

[1m[33mwarning[0m[1m: field `flags` is never read[0m
  [1m[94m--> [0mkernel/src/user.rs:87:9
   [1m[94m|[0m
[1m[94m82[0m [1m[94m|[0m pub struct ElfSegment {
   [1m[94m|[0m            [1m[94m----------[0m [1m[94mfield in this struct[0m
[1m[94m...[0m
[1m[94m87[0m [1m[94m|[0m     pub flags: u32,
   [1m[94m|[0m         [1m[33m^^^^^[0m
   [1m[94m|[0m
   [1m[94m= [0m[1mnote[0m: `ElfSegment` has derived impls for the traits `Debug` and `Clone`, but these are intentionally ignored during dead code analysis

[1mFor more information about this error, try `rustc --explain E0133`.[0m
[1m[33mwarning[0m: `kernel` (bin "kernel") generated 41 warnings (run `cargo fix --bin "kernel" -p kernel` to apply 3 suggestions)
[1m[92m    Finished[0m `release` profile [optimized] target(s) in 0.02s
[1m[33mwarning[0m[1m: unnecessary `unsafe` block[0m
  [1m[94m--> [0mkernel/src/apic.rs:54:20
   [1m[94m|[0m
[1m[94m54[0m [1m[94m|[0m         let leaf = unsafe { core::arch::x86_64::__cpuid(1) };
   [1m[94m|[0m                    [1m[33m^^^^^^[0m [1m[33munnecessary `unsafe` block[0m
   [1m[94m|[0m
   [1m[94m= [0m[1mnote[0m: `#[warn(unused_unsafe)]` (part of `#[warn(unused)]`) on by default

[1m[33mwarning[E0133][0m[1m: use of mutable static is unsafe and requires unsafe block[0m
  [1m[94m--> [0mkernel/src/heap.rs:18:41
   [1m[94m|[0m
[1m[94m18[0m [1m[94m|[0m     let start = core::ptr::addr_of_mut!(HEAP_STORAGE.0) as *mut u8;
   [1m[94m|[0m                                         [1m[33m^^^^^^^^^^^^[0m [1m[33muse of mutable static[0m
   [1m[94m|[0m
   [1m[94m= [0m[1mnote[0m: mutable statics can be mutated by multiple threads: aliasing violations or data races will cause undefined behavior
[1m[92mnote[0m: an unsafe function restricts its caller, but its body is safe by default
  [1m[94m--> [0mkernel/src/heap.rs:17:1
   [1m[94m|[0m
[1m[94m17[0m [1m[94m|[0m pub unsafe fn init() {
   [1m[94m|[0m [1m[92m^^^^^^^^^^^^^^^^^^^^[0m
   [1m[94m= [0m[1mnote[0m: for more information, see <https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-op-in-unsafe-fn.html>
   [1m[94m= [0m[1mnote[0m: `#[warn(unsafe_op_in_unsafe_fn)]` (part of `#[warn(rust_2024_compatibility)]`) on by default

[1m[33mwarning[E0133][0m[1m: call to unsafe function `memory::active_level_4_table` is unsafe and requires unsafe block[0m
   [1m[94m--> [0mkernel/src/memory.rs:131:25
    [1m[94m|[0m
[1m[94m131[0m [1m[94m|[0m     let level_4_table = active_level_4_table(physical_memory_offset);
    [1m[94m|[0m                         [1m[33m^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^[0m [1m[33mcall to unsafe function[0m
    [1m[94m|[0m
    [1m[94m= [0m[1mnote[0m: consult the function's documentation for information on how to avoid undefined behavior
[1m[92mnote[0m: an unsafe function restricts its caller, but its body is safe by default
   [1m[94m--> [0mkernel/src/memory.rs:130:1
    [1m[94m|[0m
[1m[94m130[0m [1m[94m|[0m pub unsafe fn init_offset_page_table(physical_memory_offset: VirtAddr) -> OffsetPageTable<'static> {
    [1m[94m|[0m [1m[92m^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^[0m
    [1m[94m= [0m[1mnote[0m: for more information, see <https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-op-in-unsafe-fn.html>

[1m[33mwarning[E0133][0m[1m: call to unsafe function `x86_64::structures::paging::OffsetPageTable::<'a>::new` is unsafe and requires unsafe block[0m
   [1m[94m--> [0mkernel/src/memory.rs:132:5
    [1m[94m|[0m
[1m[94m132[0m [1m[94m|[0m     OffsetPageTable::new(level_4_table, physical_memory_offset)
    [1m[94m|[0m     [1m[33m^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^[0m [1m[33mcall to unsafe function[0m
    [1m[94m|[0m
    [1m[94m= [0m[1mnote[0m: consult the function's documentation for information on how to avoid undefined behavior
    [1m[94m= [0m[1mnote[0m: for more information, see <https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-op-in-unsafe-fn.html>

[1m[33mwarning[E0133][0m[1m: call to unsafe function `x86_64::structures::paging::OffsetPageTable::<'a>::new` is unsafe and requires unsafe block[0m
   [1m[94m--> [0mkernel/src/memory.rs:200:22
    [1m[94m|[0m
[1m[94m200[0m [1m[94m|[0m     let mut mapper = OffsetPageTable::new(root_table, offset);
    [1m[94m|[0m                      [1m[33m^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^[0m [1m[33mcall to unsafe function[0m
    [1m[94m|[0m
    [1m[94m= [0m[1mnote[0m: consult the function's documentation for information on how to avoid undefined behavior
[1m[92mnote[0m: an unsafe function restricts its caller, but its body is safe by default
   [1m[94m--> [0mkernel/src/memory.rs:191:1
    [1m[94m|[0m
[1m[94m191[0m [1m[94m|[0m [1m[92m/[0m unsafe fn map_user_pages(
[1m[94m192[0m [1m[94m|[0m [1m[92m|[0m     root: PhysFrame<Size4KiB>,
[1m[94m193[0m [1m[94m|[0m [1m[92m|[0m     code: PhysFrame<Size4KiB>,
[1m[94m194[0m [1m[94m|[0m [1m[92m|[0m     stack: PhysFrame<Size4KiB>,
[1m[94m195[0m [1m[94m|[0m [1m[92m|[0m     offset: VirtAddr,
[1m[94m196[0m [1m[94m|[0m [1m[92m|[0m     allocator: &mut BootInfoFrameAllocator,
[1m[94m197[0m [1m[94m|[0m [1m[92m|[0m ) -> bool {
    [1m[94m|[0m [1m[92m|_________^[0m
    [1m[94m= [0m[1mnote[0m: for more information, see <https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-op-in-unsafe-fn.html>

[1m[33mwarning[E0133][0m[1m: call to unsafe function `x86_64::structures::paging::Mapper::map_to` is unsafe and requires unsafe block[0m
   [1m[94m--> [0mkernel/src/memory.rs:204:20
    [1m[94m|[0m
[1m[94m204[0m [1m[94m|[0m     let code_map = mapper.map_to(code_page, code, flags, allocator);
    [1m[94m|[0m                    [1m[33m^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^[0m [1m[33mcall to unsafe function[0m
    [1m[94m|[0m
    [1m[94m= [0m[1mnote[0m: consult the function's documentation for information on how to avoid undefined behavior
    [1m[94m= [0m[1mnote[0m: for more information, see <https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-op-in-unsafe-fn.html>

[1m[33mwarning[E0133][0m[1m: call to unsafe function `x86_64::structures::paging::Mapper::map_to` is unsafe and requires unsafe block[0m
   [1m[94m--> [0mkernel/src/memory.rs:205:21
    [1m[94m|[0m
[1m[94m205[0m [1m[94m|[0m       let stack_map = mapper.map_to(
    [1m[94m|[0m [1m[33m _____________________^[0m
[1m[94m206[0m [1m[94m|[0m [1m[33m|[0m         stack_page,
[1m[94m207[0m [1m[94m|[0m [1m[33m|[0m         stack,
[1m[94m208[0m [1m[94m|[0m [1m[33m|[0m         flags | PageTableFlags::WRITABLE,
[1m[94m209[0m [1m[94m|[0m [1m[33m|[0m         allocator,
[1m[94m210[0m [1m[94m|[0m [1m[33m|[0m     );
    [1m[94m|[0m [1m[33m|_____^[0m [1m[33mcall to unsafe function[0m
    [1m[94m|[0m
    [1m[94m= [0m[1mnote[0m: consult the function's documentation for information on how to avoid undefined behavior
    [1m[94m= [0m[1mnote[0m: for more information, see <https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-op-in-unsafe-fn.html>

[1m[33mwarning[0m[1m: variants `MissingRsdp` and `ShortTable` are never constructed[0m
  [1m[94m--> [0mkernel/src/acpi.rs:13:5
   [1m[94m|[0m
[1m[94m12[0m [1m[94m|[0m pub enum AcpiError {
   [1m[94m|[0m          [1m[94m---------[0m [1m[94mvariants in this enum[0m
[1m[94m13[0m [1m[94m|[0m     MissingRsdp,
   [1m[94m|[0m     [1m[33m^^^^^^^^^^^[0m
[1m[94m...[0m
[1m[94m16[0m [1m[94m|[0m     ShortTable,
   [1m[94m|[0m     [1m[33m^^^^^^^^^^[0m
   [1m[94m|[0m
   [1m[94m= [0m[1mnote[0m: `AcpiError` has derived impls for the traits `Clone` and `Debug`, but these are intentionally ignored during dead code analysis
   [1m[94m= [0m[1mnote[0m: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default

[1m[33mwarning[0m[1m: constant `LAPIC_ICR_LOW` is never used[0m
  [1m[94m--> [0mkernel/src/apic.rs:17:7
   [1m[94m|[0m
[1m[94m17[0m [1m[94m|[0m const LAPIC_ICR_LOW: u64 = 0x300;
   [1m[94m|[0m       [1m[33m^^^^^^^^^^^^^[0m

[1m[33mwarning[0m[1m: constant `LAPIC_ICR_HIGH` is never used[0m
  [1m[94m--> [0mkernel/src/apic.rs:18:7
   [1m[94m|[0m
[1m[94m18[0m [1m[94m|[0m const LAPIC_ICR_HIGH: u64 = 0x310;
   [1m[94m|[0m       [1m[33m^^^^^^^^^^^^^^[0m

[1m[33mwarning[0m[1m: function `start_ap` is never used[0m
   [1m[94m--> [0mkernel/src/apic.rs:132:8
    [1m[94m|[0m
[1m[94m132[0m [1m[94m|[0m pub fn start_ap(local_id: u32, startup_vector: u8) -> bool {
    [1m[94m|[0m        [1m[33m^^^^^^^^[0m

[1m[33mwarning[0m[1m: function `wait_for_icr_idle` is never used[0m
   [1m[94m--> [0mkernel/src/apic.rs:159:4
    [1m[94m|[0m
[1m[94m159[0m [1m[94m|[0m fn wait_for_icr_idle() -> bool {
    [1m[94m|[0m    [1m[33m^^^^^^^^^^^^^^^^^[0m

[1m[33mwarning[0m[1m: function `build_idt_at` is never used[0m
  [1m[94m--> [0mkernel/src/interrupts.rs:85:15
   [1m[94m|[0m
[1m[94m85[0m [1m[94m|[0m pub unsafe fn build_idt_at(address: VirtAddr) -> bool {
   [1m[94m|[0m               [1m[33m^^^^^^^^^^^^[0m

[1m[33mwarning[0m[1m: constant `SLOT_BITS` is never used[0m
 [1m[94m--> [0mkernel/src/ipc.rs:6:7
  [1m[94m|[0m
[1m[94m6[0m [1m[94m|[0m const SLOT_BITS: u32 = 3;
  [1m[94m|[0m       [1m[33m^^^^^^^^^[0m

[1m[33mwarning[0m[1m: method `allocate_frame_below` is never used[0m
  [1m[94m--> [0mkernel/src/memory.rs:65:12
   [1m[94m|[0m
[1m[94m23[0m [1m[94m|[0m impl BootInfoFrameAllocator {
   [1m[94m|[0m [1m[94m---------------------------[0m [1m[94mmethod in this implementation[0m
[1m[94m...[0m
[1m[94m65[0m [1m[94m|[0m     pub fn allocate_frame_below(&mut self, limit: u64) -> Option<PhysFra[1m[94m...[0m
   [1m[94m|[0m            [1m[33m^^^^^^^^^^^^^^^^^^^^[0m

[1m[33mwarning[0m[1m: constant `TRAMPOLINE_LIMIT` is never used[0m
  [1m[94m--> [0mkernel/src/smp.rs:13:7
   [1m[94m|[0m
[1m[94m13[0m [1m[94m|[0m const TRAMPOLINE_LIMIT: u64 = 0x10_0000;
   [1m[94m|[0m       [1m[33m^^^^^^^^^^^^^^^^[0m

[1m[33mwarning[0m[1m: constant `PML4_SLOT` is never used[0m
  [1m[94m--> [0mkernel/src/smp.rs:14:7
   [1m[94m|[0m
[1m[94m14[0m [1m[94m|[0m const PML4_SLOT: usize = 16;
   [1m[94m|[0m       [1m[33m^^^^^^^^^[0m

[1m[33mwarning[0m[1m: constant `STACK_SLOT` is never used[0m
  [1m[94m--> [0mkernel/src/smp.rs:15:7
   [1m[94m|[0m
[1m[94m15[0m [1m[94m|[0m const STACK_SLOT: usize = 24;
   [1m[94m|[0m       [1m[33m^^^^^^^^^^[0m

[1m[33mwarning[0m[1m: constant `CPU_SLOT` is never used[0m
  [1m[94m--> [0mkernel/src/smp.rs:16:7
   [1m[94m|[0m
[1m[94m16[0m [1m[94m|[0m const CPU_SLOT: usize = 32;
   [1m[94m|[0m       [1m[33m^^^^^^^^[0m

[1m[33mwarning[0m[1m: constant `ENTRY_SLOT` is never used[0m
  [1m[94m--> [0mkernel/src/smp.rs:17:7
   [1m[94m|[0m
[1m[94m17[0m [1m[94m|[0m const ENTRY_SLOT: usize = 40;
   [1m[94m|[0m       [1m[33m^^^^^^^^^^[0m

[1m[33mwarning[0m[1m: constant `LONG_ENTRY_SLOT` is never used[0m
  [1m[94m--> [0mkernel/src/smp.rs:18:7
   [1m[94m|[0m
[1m[94m18[0m [1m[94m|[0m const LONG_ENTRY_SLOT: usize = 48;
   [1m[94m|[0m       [1m[33m^^^^^^^^^^^^^^^[0m

[1m[33mwarning[0m[1m: constant `ONLINE_SLOT` is never used[0m
  [1m[94m--> [0mkernel/src/smp.rs:19:7
   [1m[94m|[0m
[1m[94m19[0m [1m[94m|[0m const ONLINE_SLOT: usize = 56;
   [1m[94m|[0m       [1m[33m^^^^^^^^^^^[0m

[1m[33mwarning[0m[1m: constant `IDT_SLOT` is never used[0m
  [1m[94m--> [0mkernel/src/smp.rs:20:7
   [1m[94m|[0m
[1m[94m20[0m [1m[94m|[0m const IDT_SLOT: usize = 64;
   [1m[94m|[0m       [1m[33m^^^^^^^^[0m

[1m[33mwarning[0m[1m: constant `STAGE_SLOT` is never used[0m
  [1m[94m--> [0mkernel/src/smp.rs:21:7
   [1m[94m|[0m
[1m[94m21[0m [1m[94m|[0m const STAGE_SLOT: usize = 72;
   [1m[94m|[0m       [1m[33m^^^^^^^^^^[0m

[1m[33mwarning[0m[1m: constant `PER_CPU_SLOT` is never used[0m
  [1m[94m--> [0mkernel/src/smp.rs:22:7
   [1m[94m|[0m
[1m[94m22[0m [1m[94m|[0m const PER_CPU_SLOT: usize = 80;
   [1m[94m|[0m       [1m[33m^^^^^^^^^^^^[0m

[1m[33mwarning[0m[1m: constant `LONG_MODE_OFFSET` is never used[0m
  [1m[94m--> [0mkernel/src/smp.rs:23:7
   [1m[94m|[0m
[1m[94m23[0m [1m[94m|[0m const LONG_MODE_OFFSET: u64 = 0x180;
   [1m[94m|[0m       [1m[33m^^^^^^^^^^^^^^^^[0m

[1m[33mwarning[0m[1m: constant `PER_CPU_OFFLINE` is never used[0m
  [1m[94m--> [0mkernel/src/smp.rs:25:7
   [1m[94m|[0m
[1m[94m25[0m [1m[94m|[0m const PER_CPU_OFFLINE: u8 = 0;
   [1m[94m|[0m       [1m[33m^^^^^^^^^^^^^^^[0m

[1m[33mwarning[0m[1m: constant `PER_CPU_LONG_MODE` is never used[0m
  [1m[94m--> [0mkernel/src/smp.rs:26:7
   [1m[94m|[0m
[1m[94m26[0m [1m[94m|[0m const PER_CPU_LONG_MODE: u8 = 1;
   [1m[94m|[0m       [1m[33m^^^^^^^^^^^^^^^^^[0m

[1m[33mwarning[0m[1m: associated items `new` and `reset` are never used[0m
  [1m[94m--> [0mkernel/src/smp.rs:46:14
   [1m[94m|[0m
[1m[94m45[0m [1m[94m|[0m impl PerCpuState {
   [1m[94m|[0m [1m[94m----------------[0m [1m[94massociated items in this implementation[0m
[1m[94m46[0m [1m[94m|[0m     const fn new() -> Self {
   [1m[94m|[0m              [1m[33m^^^[0m
[1m[94m...[0m
[1m[94m61[0m [1m[94m|[0m     fn reset(&self, self_base: u64, apic_id: u64, stack_top: u64) {
   [1m[94m|[0m        [1m[33m^^^^^[0m

[1m[33mwarning[0m[1m: struct `PerCpuSnapshot` is never constructed[0m
  [1m[94m--> [0mkernel/src/smp.rs:89:12
   [1m[94m|[0m
[1m[94m89[0m [1m[94m|[0m pub struct PerCpuSnapshot {
   [1m[94m|[0m            [1m[33m^^^^^^^^^^^^^^[0m

[1m[33mwarning[0m[1m: function `per_cpu_snapshot` is never used[0m
   [1m[94m--> [0mkernel/src/smp.rs:103:15
    [1m[94m|[0m
[1m[94m103[0m [1m[94m|[0m pub unsafe fn per_cpu_snapshot(base: u64, index: usize) -> Option<PerCp[1m[94m...[0m
    [1m[94m|[0m               [1m[33m^^^^^^^^^^^^^^^^[0m

[1m[33mwarning[0m[1m: function `initialize_per_cpu_frame` is never used[0m
   [1m[94m--> [0mkernel/src/smp.rs:128:11
    [1m[94m|[0m
[1m[94m128[0m [1m[94m|[0m unsafe fn initialize_per_cpu_frame(base: u64) {
    [1m[94m|[0m           [1m[33m^^^^^^^^^^^^^^^^^^^^^^^^[0m

[1m[33mwarning[0m[1m: function `count_per_cpu_online` is never used[0m
   [1m[94m--> [0mkernel/src/smp.rs:196:4
    [1m[94m|[0m
[1m[94m196[0m [1m[94m|[0m fn count_per_cpu_online(base: u64, limit: usize) -> u32 {
    [1m[94m|[0m    [1m[33m^^^^^^^^^^^^^^^^^^^^[0m

[1m[33mwarning[0m[1m: function `bring_up` is never used[0m
   [1m[94m--> [0mkernel/src/smp.rs:236:8
    [1m[94m|[0m
[1m[94m236[0m [1m[94m|[0m pub fn bring_up(
    [1m[94m|[0m        [1m[33m^^^^^^^^[0m

[1m[33mwarning[0m[1m: function `ensure_kernel_coverage` is never used[0m
   [1m[94m--> [0mkernel/src/smp.rs:548:4
    [1m[94m|[0m
[1m[94m548[0m [1m[94m|[0m fn ensure_kernel_coverage(
    [1m[94m|[0m    [1m[33m^^^^^^^^^^^^^^^^^^^^^^[0m

[1m[33mwarning[0m[1m: function `install_trampoline` is never used[0m
   [1m[94m--> [0mkernel/src/smp.rs:593:4
    [1m[94m|[0m
[1m[94m593[0m [1m[94m|[0m fn install_trampoline(physical: u64, offset: VirtAddr) -> bool {
    [1m[94m|[0m    [1m[33m^^^^^^^^^^^^^^^^^^[0m

[1m[33mwarning[0m[1m: function `patch_trampoline` is never used[0m
   [1m[94m--> [0mkernel/src/smp.rs:609:4
    [1m[94m|[0m
[1m[94m609[0m [1m[94m|[0m fn patch_trampoline(
    [1m[94m|[0m    [1m[33m^^^^^^^^^^^^^^^^[0m

[1m[33mwarning[0m[1m: function `wait_online` is never used[0m
   [1m[94m--> [0mkernel/src/smp.rs:641:4
    [1m[94m|[0m
[1m[94m641[0m [1m[94m|[0m fn wait_online(offset: VirtAddr, online_physical: u64, _index: usize) -[1m[94m...[0m
    [1m[94m|[0m    [1m[33m^^^^^^^^^^^[0m

[1m[33mwarning[0m[1m: variant `Exited` is never constructed[0m
  [1m[94m--> [0mkernel/src/syscall.rs:28:5
   [1m[94m|[0m
[1m[94m26[0m [1m[94m|[0m pub enum Return {
   [1m[94m|[0m          [1m[94m------[0m [1m[94mvariant in this enum[0m
[1m[94m27[0m [1m[94m|[0m     Value(u64),
[1m[94m28[0m [1m[94m|[0m     Exited(u64),
   [1m[94m|[0m     [1m[33m^^^^^^[0m
   [1m[94m|[0m
   [1m[94m= [0m[1mnote[0m: `Return` has derived impls for the traits `Debug` and `Clone`, but these are intentionally ignored during dead code analysis

[1m[33mwarning[0m[1m: fields `code` and `stack` are never read[0m
  [1m[94m--> [0mkernel/src/user.rs:15:9
   [1m[94m|[0m
[1m[94m14[0m [1m[94m|[0m pub struct UserMapping {
   [1m[94m|[0m            [1m[94m-----------[0m [1m[94mfields in this struct[0m
[1m[94m15[0m [1m[94m|[0m     pub code: PhysFrame<Size4KiB>,
   [1m[94m|[0m         [1m[33m^^^^[0m
[1m[94m16[0m [1m[94m|[0m     pub stack: PhysFrame<Size4KiB>,
   [1m[94m|[0m         [1m[33m^^^^^[0m
   [1m[94m|[0m
   [1m[94m= [0m[1mnote[0m: `UserMapping` has a derived impl for the trait `Clone`, but this is intentionally ignored during dead code analysis

[1m[33mwarning[0m[1m: field `flags` is never read[0m
  [1m[94m--> [0mkernel/src/user.rs:87:9
   [1m[94m|[0m
[1m[94m82[0m [1m[94m|[0m pub struct ElfSegment {
   [1m[94m|[0m            [1m[94m----------[0m [1m[94mfield in this struct[0m
[1m[94m...[0m
[1m[94m87[0m [1m[94m|[0m     pub flags: u32,
   [1m[94m|[0m         [1m[33m^^^^^[0m
   [1m[94m|[0m
   [1m[94m= [0m[1mnote[0m: `ElfSegment` has derived impls for the traits `Debug` and `Clone`, but these are intentionally ignored during dead code analysis

[1mFor more information about this error, try `rustc --explain E0133`.[0m
[1m[33mwarning[0m: `kernel` (bin "kernel") generated 41 warnings (run `cargo fix --bin "kernel" -p kernel` to apply 3 suggestions)
[1m[92m    Finished[0m `release` profile [optimized] target(s) in 0.02s
INFO : Framebuffer info: FrameBufferInfo { byte_len: 2764800, width: 1280, height: 720, pixel_format: Bgr, bytes_per_pixel: 3, stride: 1280 }
INFO : 4th Stage
INFO : BiosInfo { stage_4: Region { start: 130000, len: 25208 }, kernel: Region { start: 1000000, len: 83b68 }, ramdisk: Region { start: 1084000, len: 0 }, config_file: Region { start: 1084000, len: 0 }, last_used_addr: 1083fff, framebuffer: BiosFramebufferInfo { region: Region { start: fd000000, len: 2a3000 }, width: 500, height: 2d0, bytes_per_pixel: 3, stride: 500, pixel_format: Bgr }, memory_map_addr: 12148, memory_map_len: 7 }
INFO : BIOS boot
INFO : Elf file loaded at Pointer {
    addr: 0x0000000001000000,
    metadata: 539496,
}
INFO : virtual_address_offset: 0x10000000000
INFO : Handling Segment: Ph64(ProgramHeader64 { type_: Ok(Load), flags: Flags(4), offset: 0, virtual_addr: 0, physical_addr: 0, file_size: 1360c, mem_size: 1360c, align: 1000 })
INFO : Handling Segment: Ph64(ProgramHeader64 { type_: Ok(Load), flags: Flags(5), offset: 13610, virtual_addr: 14610, physical_addr: 14610, file_size: 3477f, mem_size: 3477f, align: 1000 })
INFO : Handling Segment: Ph64(ProgramHeader64 { type_: Ok(Load), flags: Flags(6), offset: 47d90, virtual_addr: 49d90, physical_addr: 49d90, file_size: 27e0, mem_size: 3270, align: 1000 })
INFO : Mapping bss section
INFO : Handling Segment: Ph64(ProgramHeader64 { type_: Ok(Load), flags: Flags(6), offset: 4a570, virtual_addr: 4d570, physical_addr: 4d570, file_size: ba8, mem_size: 318a1, align: 1000 })
INFO : Mapping bss section
INFO : Entry point at: 0x10000022f00
INFO : Creating GDT at PhysAddr(0x10d7000)
INFO : Map framebuffer
INFO : Map physical memory
INFO : Allocate bootinfo
INFO : Create Memory Map
INFO : Create bootinfo
INFO : Jumping to kernel entry point at VirtAddr(0x10000022f00)
NOVEL_OS :: PHASE 3 MEMORY MANAGEMENT
bootloader API: ApiVersion { version_major: 0, version_minor: 11, version_patch: 17, pre_release: false }
GDT user selectors: data=SegmentSelector { index: 3, rpl: Ring3 } code=SegmentSelector { index: 4, rpl: Ring3 }
GDT + TSS + IDT + PIC loaded; APIC xAPIC=true x2APIC=false id=0
IDT ready; proceeding to memory setup
starting physical memory setup
physical memory offset: VirtAddr(0x28000000000)
ACPI: rsdp=true xsdt=false madt=true cpus=1 ioapics=1 lapic=0xfee00000 ioapic=0xfec00000
APIC routing: lapic=true ioapic=true timer_vector=32 keyboard_vector=33 local_id=0
usable frames discovered: 31386
frame recycle invariant: true
SMP address base: kernel_offset=0x10000000000 ap_entry_runtime=0x1000003c090
creating offset page-table mapper
SMP: discovered=1 attempted=0 online=1 trampoline=false
active level-4 page table mapped
user mappings: code=0x400000 stack=0x7ff000 guard=0x7fe000 valid=true
mapping heap pages
heap pages mapped: 25
heap translation: true
user mapping permissions: true
process address space CR3 isolation: true
process teardown: first=true second=true
global allocator initialized
ELF loader PT_LOAD+BSS: true
memory: 31386 usable frames, 25 heap pages
paging + heap ready: Vec capacity=64, checksum=34464
scheduler smoke: completed=2, pending=0, runs=2
context switch smoke: true
VFS smoke: true, handles: true, ramdisk: true, global=true
syscall ABI smoke: true
IPC smoke: ports=true generation=true ownership=true
capability table: namespace=true opaque=true generation=true revoked=true
loopback: smoke=true checksum=true backpressure=true
loopback datagram: roundtrip=true ports=1000->2000 checksum=true corruption_rejected=true
block device: ata_pio_sector0=true mbr_partitions=true
persistent fs: novelfs_read=true write_rollback=true
user memory validation: true
ELF parser validation: true
init ELF from ramdisk: fs=true loaded=true bytes=347
process table: init registered=true current=true state=Running worker=2 state=Ready worker=3 state=Ready
exception table ready; starting timer bring-up
hardware IRQs online: 4 timer ticks
keyboard queue ready: no pending scancode
preemptive scheduler: task0=4 task1=4 switches=9 starvation=false
EXCEPTION: BREAKPOINT (#BP) handled
breakpoint exception works after hardware IRQs
user preemption: contexts=3 timer_frame=true
RING3 transition: init ELF iretq submitted
RING3 int80: cpl=3 valid=true
init from elf
SYSCALL write: accepted length=14
RING3 int80: cpl=3 valid=true
SYSCALL open: handle=1
RING3 int80: cpl=3 valid=true
SYSCALL read: handle=1 bytes=14
RING3 int80: cpl=3 valid=true
NOVEL_OS INIT
SYSCALL write: accepted length=14
RING3 int80: cpl=3 valid=true
SYSCALL close: handle=1 closed=true
RING3 int80: cpl=3 valid=true
SYSCALL port_create: handle=2
RING3 int80: cpl=3 valid=true
SYSCALL port_send: handle=2 bytes=3
RING3 int80: cpl=3 valid=true
SYSCALL port_recv: handle=2 bytes=3
RING3 int80: cpl=3 valid=true
init from elf
SYSCALL write: accepted length=14
RING3 int80: cpl=3 valid=true
SYSCALL open: handle=4194305
RING3 int80: cpl=3 valid=true
SYSCALL read: handle=4194305 bytes=14
RING3 int80: cpl=3 valid=true
NOVEL_OS INIT
SYSCALL write: accepted length=14
RING3 int80: cpl=3 valid=true
SYSCALL close: handle=4194305 closed=true
RING3 int80: cpl=3 valid=true
SYSCALL port_create: handle=257
RING3 int80: cpl=3 valid=true
SYSCALL port_send: handle=257 bytes=3
RING3 int80: cpl=3 valid=true
SYSCALL port_recv: handle=257 bytes=3
RING3 int80: cpl=3 valid=true
exit from elf
SYSCALL write: accepted length=14
RING3 int80: cpl=3 valid=true
SYSCALL exit: handoff status=0
user preemption: switches=2 frames=true starvation=false
PROCESS EXIT: status=0 owner=3 state=Exited revoked_handles=0 revoked_ports=0 reclaimed=true next=true
PROCESS HANDOFF: from=3 to=1 state=Running cr3=switched active=true resume=saved
USER FAULT POLICY: kill=true status=61444 address=Ok(VirtAddr(0x7fe000)) error=PageFaultErrorCode(USER_MODE)
user preemption: switches=2 frames=true starvation=false
PROCESS EXIT: status=61444 owner=1 state=Faulted revoked_handles=0 revoked_ports=1 reclaimed=true next=true
PROCESS HANDOFF: from=1 to=2 state=Running cr3=switched active=true resume=saved
USER FAULT POLICY: kill=true status=61444 address=Ok(VirtAddr(0x7fe000)) error=PageFaultErrorCode(USER_MODE)
user preemption: switches=2 frames=true starvation=false
PROCESS EXIT: status=61444 owner=2 state=Faulted revoked_handles=0 revoked_ports=1 reclaimed=true next=false
USER QUEUE: idle=true runnable=0
release gate: PASS
[1m[33mwarning[0m[1m: unnecessary `unsafe` block[0m
  [1m[94m--> [0mkernel/src/apic.rs:54:20
   [1m[94m|[0m
[1m[94m54[0m [1m[94m|[0m         let leaf = unsafe { core::arch::x86_64::__cpuid(1) };
   [1m[94m|[0m                    [1m[33m^^^^^^[0m [1m[33munnecessary `unsafe` block[0m
   [1m[94m|[0m
   [1m[94m= [0m[1mnote[0m: `#[warn(unused_unsafe)]` (part of `#[warn(unused)]`) on by default

[1m[33mwarning[E0133][0m[1m: use of mutable static is unsafe and requires unsafe block[0m
  [1m[94m--> [0mkernel/src/heap.rs:18:41
   [1m[94m|[0m
[1m[94m18[0m [1m[94m|[0m     let start = core::ptr::addr_of_mut!(HEAP_STORAGE.0) as *mut u8;
   [1m[94m|[0m                                         [1m[33m^^^^^^^^^^^^[0m [1m[33muse of mutable static[0m
   [1m[94m|[0m
   [1m[94m= [0m[1mnote[0m: mutable statics can be mutated by multiple threads: aliasing violations or data races will cause undefined behavior
[1m[92mnote[0m: an unsafe function restricts its caller, but its body is safe by default
  [1m[94m--> [0mkernel/src/heap.rs:17:1
   [1m[94m|[0m
[1m[94m17[0m [1m[94m|[0m pub unsafe fn init() {
   [1m[94m|[0m [1m[92m^^^^^^^^^^^^^^^^^^^^[0m
   [1m[94m= [0m[1mnote[0m: for more information, see <https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-op-in-unsafe-fn.html>
   [1m[94m= [0m[1mnote[0m: `#[warn(unsafe_op_in_unsafe_fn)]` (part of `#[warn(rust_2024_compatibility)]`) on by default

[1m[33mwarning[E0133][0m[1m: call to unsafe function `memory::active_level_4_table` is unsafe and requires unsafe block[0m
   [1m[94m--> [0mkernel/src/memory.rs:131:25
    [1m[94m|[0m
[1m[94m131[0m [1m[94m|[0m     let level_4_table = active_level_4_table(physical_memory_offset);
    [1m[94m|[0m                         [1m[33m^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^[0m [1m[33mcall to unsafe function[0m
    [1m[94m|[0m
    [1m[94m= [0m[1mnote[0m: consult the function's documentation for information on how to avoid undefined behavior
[1m[92mnote[0m: an unsafe function restricts its caller, but its body is safe by default
   [1m[94m--> [0mkernel/src/memory.rs:130:1
    [1m[94m|[0m
[1m[94m130[0m [1m[94m|[0m pub unsafe fn init_offset_page_table(physical_memory_offset: VirtAddr) -> OffsetPageTable<'static> {
    [1m[94m|[0m [1m[92m^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^[0m
    [1m[94m= [0m[1mnote[0m: for more information, see <https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-op-in-unsafe-fn.html>

[1m[33mwarning[E0133][0m[1m: call to unsafe function `x86_64::structures::paging::OffsetPageTable::<'a>::new` is unsafe and requires unsafe block[0m
   [1m[94m--> [0mkernel/src/memory.rs:132:5
    [1m[94m|[0m
[1m[94m132[0m [1m[94m|[0m     OffsetPageTable::new(level_4_table, physical_memory_offset)
    [1m[94m|[0m     [1m[33m^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^[0m [1m[33mcall to unsafe function[0m
    [1m[94m|[0m
    [1m[94m= [0m[1mnote[0m: consult the function's documentation for information on how to avoid undefined behavior
    [1m[94m= [0m[1mnote[0m: for more information, see <https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-op-in-unsafe-fn.html>

[1m[33mwarning[E0133][0m[1m: call to unsafe function `x86_64::structures::paging::OffsetPageTable::<'a>::new` is unsafe and requires unsafe block[0m
   [1m[94m--> [0mkernel/src/memory.rs:200:22
    [1m[94m|[0m
[1m[94m200[0m [1m[94m|[0m     let mut mapper = OffsetPageTable::new(root_table, offset);
    [1m[94m|[0m                      [1m[33m^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^[0m [1m[33mcall to unsafe function[0m
    [1m[94m|[0m
    [1m[94m= [0m[1mnote[0m: consult the function's documentation for information on how to avoid undefined behavior
[1m[92mnote[0m: an unsafe function restricts its caller, but its body is safe by default
   [1m[94m--> [0mkernel/src/memory.rs:191:1
    [1m[94m|[0m
[1m[94m191[0m [1m[94m|[0m [1m[92m/[0m unsafe fn map_user_pages(
[1m[94m192[0m [1m[94m|[0m [1m[92m|[0m     root: PhysFrame<Size4KiB>,
[1m[94m193[0m [1m[94m|[0m [1m[92m|[0m     code: PhysFrame<Size4KiB>,
[1m[94m194[0m [1m[94m|[0m [1m[92m|[0m     stack: PhysFrame<Size4KiB>,
[1m[94m195[0m [1m[94m|[0m [1m[92m|[0m     offset: VirtAddr,
[1m[94m196[0m [1m[94m|[0m [1m[92m|[0m     allocator: &mut BootInfoFrameAllocator,
[1m[94m197[0m [1m[94m|[0m [1m[92m|[0m ) -> bool {
    [1m[94m|[0m [1m[92m|_________^[0m
    [1m[94m= [0m[1mnote[0m: for more information, see <https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-op-in-unsafe-fn.html>

[1m[33mwarning[E0133][0m[1m: call to unsafe function `x86_64::structures::paging::Mapper::map_to` is unsafe and requires unsafe block[0m
   [1m[94m--> [0mkernel/src/memory.rs:204:20
    [1m[94m|[0m
[1m[94m204[0m [1m[94m|[0m     let code_map = mapper.map_to(code_page, code, flags, allocator);
    [1m[94m|[0m                    [1m[33m^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^[0m [1m[33mcall to unsafe function[0m
    [1m[94m|[0m
    [1m[94m= [0m[1mnote[0m: consult the function's documentation for information on how to avoid undefined behavior
    [1m[94m= [0m[1mnote[0m: for more information, see <https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-op-in-unsafe-fn.html>

[1m[33mwarning[E0133][0m[1m: call to unsafe function `x86_64::structures::paging::Mapper::map_to` is unsafe and requires unsafe block[0m
   [1m[94m--> [0mkernel/src/memory.rs:205:21
    [1m[94m|[0m
[1m[94m205[0m [1m[94m|[0m       let stack_map = mapper.map_to(
    [1m[94m|[0m [1m[33m _____________________^[0m
[1m[94m206[0m [1m[94m|[0m [1m[33m|[0m         stack_page,
[1m[94m207[0m [1m[94m|[0m [1m[33m|[0m         stack,
[1m[94m208[0m [1m[94m|[0m [1m[33m|[0m         flags | PageTableFlags::WRITABLE,
[1m[94m209[0m [1m[94m|[0m [1m[33m|[0m         allocator,
[1m[94m210[0m [1m[94m|[0m [1m[33m|[0m     );
    [1m[94m|[0m [1m[33m|_____^[0m [1m[33mcall to unsafe function[0m
    [1m[94m|[0m
    [1m[94m= [0m[1mnote[0m: consult the function's documentation for information on how to avoid undefined behavior
    [1m[94m= [0m[1mnote[0m: for more information, see <https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-op-in-unsafe-fn.html>

[1m[33mwarning[0m[1m: variants `MissingRsdp` and `ShortTable` are never constructed[0m
  [1m[94m--> [0mkernel/src/acpi.rs:13:5
   [1m[94m|[0m
[1m[94m12[0m [1m[94m|[0m pub enum AcpiError {
   [1m[94m|[0m          [1m[94m---------[0m [1m[94mvariants in this enum[0m
[1m[94m13[0m [1m[94m|[0m     MissingRsdp,
   [1m[94m|[0m     [1m[33m^^^^^^^^^^^[0m
[1m[94m...[0m
[1m[94m16[0m [1m[94m|[0m     ShortTable,
   [1m[94m|[0m     [1m[33m^^^^^^^^^^[0m
   [1m[94m|[0m
   [1m[94m= [0m[1mnote[0m: `AcpiError` has derived impls for the traits `Clone` and `Debug`, but these are intentionally ignored during dead code analysis
   [1m[94m= [0m[1mnote[0m: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default

[1m[33mwarning[0m[1m: constant `LAPIC_ICR_LOW` is never used[0m
  [1m[94m--> [0mkernel/src/apic.rs:17:7
   [1m[94m|[0m
[1m[94m17[0m [1m[94m|[0m const LAPIC_ICR_LOW: u64 = 0x300;
   [1m[94m|[0m       [1m[33m^^^^^^^^^^^^^[0m

[1m[33mwarning[0m[1m: constant `LAPIC_ICR_HIGH` is never used[0m
  [1m[94m--> [0mkernel/src/apic.rs:18:7
   [1m[94m|[0m
[1m[94m18[0m [1m[94m|[0m const LAPIC_ICR_HIGH: u64 = 0x310;
   [1m[94m|[0m       [1m[33m^^^^^^^^^^^^^^[0m

[1m[33mwarning[0m[1m: function `start_ap` is never used[0m
   [1m[94m--> [0mkernel/src/apic.rs:132:8
    [1m[94m|[0m
[1m[94m132[0m [1m[94m|[0m pub fn start_ap(local_id: u32, startup_vector: u8) -> bool {
    [1m[94m|[0m        [1m[33m^^^^^^^^[0m

[1m[33mwarning[0m[1m: function `wait_for_icr_idle` is never used[0m
   [1m[94m--> [0mkernel/src/apic.rs:159:4
    [1m[94m|[0m
[1m[94m159[0m [1m[94m|[0m fn wait_for_icr_idle() -> bool {
    [1m[94m|[0m    [1m[33m^^^^^^^^^^^^^^^^^[0m

[1m[33mwarning[0m[1m: function `build_idt_at` is never used[0m
  [1m[94m--> [0mkernel/src/interrupts.rs:85:15
   [1m[94m|[0m
[1m[94m85[0m [1m[94m|[0m pub unsafe fn build_idt_at(address: VirtAddr) -> bool {
   [1m[94m|[0m               [1m[33m^^^^^^^^^^^^[0m

[1m[33mwarning[0m[1m: constant `SLOT_BITS` is never used[0m
 [1m[94m--> [0mkernel/src/ipc.rs:6:7
  [1m[94m|[0m
[1m[94m6[0m [1m[94m|[0m const SLOT_BITS: u32 = 3;
  [1m[94m|[0m       [1m[33m^^^^^^^^^[0m

[1m[33mwarning[0m[1m: method `allocate_frame_below` is never used[0m
  [1m[94m--> [0mkernel/src/memory.rs:65:12
   [1m[94m|[0m
[1m[94m23[0m [1m[94m|[0m impl BootInfoFrameAllocator {
   [1m[94m|[0m [1m[94m---------------------------[0m [1m[94mmethod in this implementation[0m
[1m[94m...[0m
[1m[94m65[0m [1m[94m|[0m     pub fn allocate_frame_below(&mut self, limit: u64) -> Option<PhysFra[1m[94m...[0m
   [1m[94m|[0m            [1m[33m^^^^^^^^^^^^^^^^^^^^[0m

[1m[33mwarning[0m[1m: constant `TRAMPOLINE_LIMIT` is never used[0m
  [1m[94m--> [0mkernel/src/smp.rs:13:7
   [1m[94m|[0m
[1m[94m13[0m [1m[94m|[0m const TRAMPOLINE_LIMIT: u64 = 0x10_0000;
   [1m[94m|[0m       [1m[33m^^^^^^^^^^^^^^^^[0m

[1m[33mwarning[0m[1m: constant `PML4_SLOT` is never used[0m
  [1m[94m--> [0mkernel/src/smp.rs:14:7
   [1m[94m|[0m
[1m[94m14[0m [1m[94m|[0m const PML4_SLOT: usize = 16;
   [1m[94m|[0m       [1m[33m^^^^^^^^^[0m

[1m[33mwarning[0m[1m: constant `STACK_SLOT` is never used[0m
  [1m[94m--> [0mkernel/src/smp.rs:15:7
   [1m[94m|[0m
[1m[94m15[0m [1m[94m|[0m const STACK_SLOT: usize = 24;
   [1m[94m|[0m       [1m[33m^^^^^^^^^^[0m

[1m[33mwarning[0m[1m: constant `CPU_SLOT` is never used[0m
  [1m[94m--> [0mkernel/src/smp.rs:16:7
   [1m[94m|[0m
[1m[94m16[0m [1m[94m|[0m const CPU_SLOT: usize = 32;
   [1m[94m|[0m       [1m[33m^^^^^^^^[0m

[1m[33mwarning[0m[1m: constant `ENTRY_SLOT` is never used[0m
  [1m[94m--> [0mkernel/src/smp.rs:17:7
   [1m[94m|[0m
[1m[94m17[0m [1m[94m|[0m const ENTRY_SLOT: usize = 40;
   [1m[94m|[0m       [1m[33m^^^^^^^^^^[0m

[1m[33mwarning[0m[1m: constant `LONG_ENTRY_SLOT` is never used[0m
  [1m[94m--> [0mkernel/src/smp.rs:18:7
   [1m[94m|[0m
[1m[94m18[0m [1m[94m|[0m const LONG_ENTRY_SLOT: usize = 48;
   [1m[94m|[0m       [1m[33m^^^^^^^^^^^^^^^[0m

[1m[33mwarning[0m[1m: constant `ONLINE_SLOT` is never used[0m
  [1m[94m--> [0mkernel/src/smp.rs:19:7
   [1m[94m|[0m
[1m[94m19[0m [1m[94m|[0m const ONLINE_SLOT: usize = 56;
   [1m[94m|[0m       [1m[33m^^^^^^^^^^^[0m

[1m[33mwarning[0m[1m: constant `IDT_SLOT` is never used[0m
  [1m[94m--> [0mkernel/src/smp.rs:20:7
   [1m[94m|[0m
[1m[94m20[0m [1m[94m|[0m const IDT_SLOT: usize = 64;
   [1m[94m|[0m       [1m[33m^^^^^^^^[0m

[1m[33mwarning[0m[1m: constant `STAGE_SLOT` is never used[0m
  [1m[94m--> [0mkernel/src/smp.rs:21:7
   [1m[94m|[0m
[1m[94m21[0m [1m[94m|[0m const STAGE_SLOT: usize = 72;
   [1m[94m|[0m       [1m[33m^^^^^^^^^^[0m

[1m[33mwarning[0m[1m: constant `PER_CPU_SLOT` is never used[0m
  [1m[94m--> [0mkernel/src/smp.rs:22:7
   [1m[94m|[0m
[1m[94m22[0m [1m[94m|[0m const PER_CPU_SLOT: usize = 80;
   [1m[94m|[0m       [1m[33m^^^^^^^^^^^^[0m

[1m[33mwarning[0m[1m: constant `LONG_MODE_OFFSET` is never used[0m
  [1m[94m--> [0mkernel/src/smp.rs:23:7
   [1m[94m|[0m
[1m[94m23[0m [1m[94m|[0m const LONG_MODE_OFFSET: u64 = 0x180;
   [1m[94m|[0m       [1m[33m^^^^^^^^^^^^^^^^[0m

[1m[33mwarning[0m[1m: constant `PER_CPU_OFFLINE` is never used[0m
  [1m[94m--> [0mkernel/src/smp.rs:25:7
   [1m[94m|[0m
[1m[94m25[0m [1m[94m|[0m const PER_CPU_OFFLINE: u8 = 0;
   [1m[94m|[0m       [1m[33m^^^^^^^^^^^^^^^[0m

[1m[33mwarning[0m[1m: constant `PER_CPU_LONG_MODE` is never used[0m
  [1m[94m--> [0mkernel/src/smp.rs:26:7
   [1m[94m|[0m
[1m[94m26[0m [1m[94m|[0m const PER_CPU_LONG_MODE: u8 = 1;
   [1m[94m|[0m       [1m[33m^^^^^^^^^^^^^^^^^[0m

[1m[33mwarning[0m[1m: associated items `new` and `reset` are never used[0m
  [1m[94m--> [0mkernel/src/smp.rs:46:14
   [1m[94m|[0m
[1m[94m45[0m [1m[94m|[0m impl PerCpuState {
   [1m[94m|[0m [1m[94m----------------[0m [1m[94massociated items in this implementation[0m
[1m[94m46[0m [1m[94m|[0m     const fn new() -> Self {
   [1m[94m|[0m              [1m[33m^^^[0m
[1m[94m...[0m
[1m[94m61[0m [1m[94m|[0m     fn reset(&self, self_base: u64, apic_id: u64, stack_top: u64) {
   [1m[94m|[0m        [1m[33m^^^^^[0m

[1m[33mwarning[0m[1m: struct `PerCpuSnapshot` is never constructed[0m
  [1m[94m--> [0mkernel/src/smp.rs:89:12
   [1m[94m|[0m
[1m[94m89[0m [1m[94m|[0m pub struct PerCpuSnapshot {
   [1m[94m|[0m            [1m[33m^^^^^^^^^^^^^^[0m

[1m[33mwarning[0m[1m: function `per_cpu_snapshot` is never used[0m
   [1m[94m--> [0mkernel/src/smp.rs:103:15
    [1m[94m|[0m
[1m[94m103[0m [1m[94m|[0m pub unsafe fn per_cpu_snapshot(base: u64, index: usize) -> Option<PerCp[1m[94m...[0m
    [1m[94m|[0m               [1m[33m^^^^^^^^^^^^^^^^[0m

[1m[33mwarning[0m[1m: function `initialize_per_cpu_frame` is never used[0m
   [1m[94m--> [0mkernel/src/smp.rs:128:11
    [1m[94m|[0m
[1m[94m128[0m [1m[94m|[0m unsafe fn initialize_per_cpu_frame(base: u64) {
    [1m[94m|[0m           [1m[33m^^^^^^^^^^^^^^^^^^^^^^^^[0m

[1m[33mwarning[0m[1m: function `count_per_cpu_online` is never used[0m
   [1m[94m--> [0mkernel/src/smp.rs:196:4
    [1m[94m|[0m
[1m[94m196[0m [1m[94m|[0m fn count_per_cpu_online(base: u64, limit: usize) -> u32 {
    [1m[94m|[0m    [1m[33m^^^^^^^^^^^^^^^^^^^^[0m

[1m[33mwarning[0m[1m: function `bring_up` is never used[0m
   [1m[94m--> [0mkernel/src/smp.rs:236:8
    [1m[94m|[0m
[1m[94m236[0m [1m[94m|[0m pub fn bring_up(
    [1m[94m|[0m        [1m[33m^^^^^^^^[0m

[1m[33mwarning[0m[1m: function `ensure_kernel_coverage` is never used[0m
   [1m[94m--> [0mkernel/src/smp.rs:548:4
    [1m[94m|[0m
[1m[94m548[0m [1m[94m|[0m fn ensure_kernel_coverage(
    [1m[94m|[0m    [1m[33m^^^^^^^^^^^^^^^^^^^^^^[0m

[1m[33mwarning[0m[1m: function `install_trampoline` is never used[0m
   [1m[94m--> [0mkernel/src/smp.rs:593:4
    [1m[94m|[0m
[1m[94m593[0m [1m[94m|[0m fn install_trampoline(physical: u64, offset: VirtAddr) -> bool {
    [1m[94m|[0m    [1m[33m^^^^^^^^^^^^^^^^^^[0m

[1m[33mwarning[0m[1m: function `patch_trampoline` is never used[0m
   [1m[94m--> [0mkernel/src/smp.rs:609:4
    [1m[94m|[0m
[1m[94m609[0m [1m[94m|[0m fn patch_trampoline(
    [1m[94m|[0m    [1m[33m^^^^^^^^^^^^^^^^[0m

[1m[33mwarning[0m[1m: function `wait_online` is never used[0m
   [1m[94m--> [0mkernel/src/smp.rs:641:4
    [1m[94m|[0m
[1m[94m641[0m [1m[94m|[0m fn wait_online(offset: VirtAddr, online_physical: u64, _index: usize) -[1m[94m...[0m
    [1m[94m|[0m    [1m[33m^^^^^^^^^^^[0m

[1m[33mwarning[0m[1m: variant `Exited` is never constructed[0m
  [1m[94m--> [0mkernel/src/syscall.rs:28:5
   [1m[94m|[0m
[1m[94m26[0m [1m[94m|[0m pub enum Return {
   [1m[94m|[0m          [1m[94m------[0m [1m[94mvariant in this enum[0m
[1m[94m27[0m [1m[94m|[0m     Value(u64),
[1m[94m28[0m [1m[94m|[0m     Exited(u64),
   [1m[94m|[0m     [1m[33m^^^^^^[0m
   [1m[94m|[0m
   [1m[94m= [0m[1mnote[0m: `Return` has derived impls for the traits `Debug` and `Clone`, but these are intentionally ignored during dead code analysis

[1m[33mwarning[0m[1m: fields `code` and `stack` are never read[0m
  [1m[94m--> [0mkernel/src/user.rs:15:9
   [1m[94m|[0m
[1m[94m14[0m [1m[94m|[0m pub struct UserMapping {
   [1m[94m|[0m            [1m[94m-----------[0m [1m[94mfields in this struct[0m
[1m[94m15[0m [1m[94m|[0m     pub code: PhysFrame<Size4KiB>,
   [1m[94m|[0m         [1m[33m^^^^[0m
[1m[94m16[0m [1m[94m|[0m     pub stack: PhysFrame<Size4KiB>,
   [1m[94m|[0m         [1m[33m^^^^^[0m
   [1m[94m|[0m
   [1m[94m= [0m[1mnote[0m: `UserMapping` has a derived impl for the trait `Clone`, but this is intentionally ignored during dead code analysis

[1m[33mwarning[0m[1m: field `flags` is never read[0m
  [1m[94m--> [0mkernel/src/user.rs:87:9
   [1m[94m|[0m
[1m[94m82[0m [1m[94m|[0m pub struct ElfSegment {
   [1m[94m|[0m            [1m[94m----------[0m [1m[94mfield in this struct[0m
[1m[94m...[0m
[1m[94m87[0m [1m[94m|[0m     pub flags: u32,
   [1m[94m|[0m         [1m[33m^^^^^[0m
   [1m[94m|[0m
   [1m[94m= [0m[1mnote[0m: `ElfSegment` has derived impls for the traits `Debug` and `Clone`, but these are intentionally ignored during dead code analysis

[1mFor more information about this error, try `rustc --explain E0133`.[0m
[1m[33mwarning[0m: `kernel` (bin "kernel") generated 41 warnings (run `cargo fix --bin "kernel" -p kernel` to apply 3 suggestions)
[1m[92m    Finished[0m `release` profile [optimized] target(s) in 0.02s
[2J[01;01H[=3h[2J[01;01H[2J[01;01H[=3h[2J[01;01H[2J[01;01H[=3h[2J[01;01HBdsDxe: loading Boot0001 "UEFI QEMU HARDDISK QM00001 " from PciRoot(0x0)/Pci(0x1F,0x2)/Sata(0x0,0xFFFF,0x0)
BdsDxe: starting Boot0001 "UEFI QEMU HARDDISK QM00001 " from PciRoot(0x0)/Pci(0x1F,0x2)/Sata(0x0,0xFFFF,0x0)
INFO : Framebuffer info: FrameBufferInfo { byte_len: 4096000, width: 1280, height: 800, pixel_format: Bgr, bytes_per_pixel: 4, stride: 1280 }
INFO : UEFI bootloader started
INFO : Using framebuffer at 0x80000000
INFO : Reading configuration from disk was successful
INFO : Trying to load ramdisk via Disk
INFO : Ramdisk not found.
TRACE: exiting boot services
TRACE: switching to new level 4 table
INFO : New page table at: PhysFrame[4KiB](0x101000)
INFO : Elf file loaded at Pointer {
    addr: 0x000000000e0c4000,
    metadata: 539496,
}
INFO : virtual_address_offset: 0x10000000000
INFO : Handling Segment: Ph64(ProgramHeader64 { type_: Ok(Load), flags: Flags(4), offset: 0, virtual_addr: 0, physical_addr: 0, file_size: 1360c, mem_size: 1360c, align: 1000 })
INFO : Handling Segment: Ph64(ProgramHeader64 { type_: Ok(Load), flags: Flags(5), offset: 13610, virtual_addr: 14610, physical_addr: 14610, file_size: 3477f, mem_size: 3477f, align: 1000 })
INFO : Handling Segment: Ph64(ProgramHeader64 { type_: Ok(Load), flags: Flags(6), offset: 47d90, virtual_addr: 49d90, physical_addr: 49d90, file_size: 27e0, mem_size: 3270, align: 1000 })
INFO : Mapping bss section
INFO : Handling Segment: Ph64(ProgramHeader64 { type_: Ok(Load), flags: Flags(6), offset: 4a570, virtual_addr: 4d570, physical_addr: 4d570, file_size: ba8, mem_size: 318a1, align: 1000 })
INFO : Mapping bss section
INFO : Entry point at: 0x10000022f00
INFO : Creating GDT at PhysAddr(0x154000)
INFO : Map framebuffer
INFO : Map physical memory
INFO : Allocate bootinfo
INFO : Create Memory Map
INFO : Create bootinfo
INFO : Jumping to kernel entry point at VirtAddr(0x10000022f00)
NOVEL_OS :: PHASE 3 MEMORY MANAGEMENT
bootloader API: ApiVersion { version_major: 0, version_minor: 11, version_patch: 17, pre_release: false }
GDT user selectors: data=SegmentSelector { index: 3, rpl: Ring3 } code=SegmentSelector { index: 4, rpl: Ring3 }
GDT + TSS + IDT + PIC loaded; APIC xAPIC=true x2APIC=false id=0
IDT ready; proceeding to memory setup
starting physical memory setup
physical memory offset: VirtAddr(0x28000000000)
ACPI: rsdp=true xsdt=true madt=true cpus=1 ioapics=1 lapic=0xfee00000 ioapic=0xfec00000
APIC routing: lapic=true ioapic=true timer_vector=32 keyboard_vector=33 local_id=0
usable frames discovered: 62689
frame recycle invariant: true
SMP address base: kernel_offset=0x10000000000 ap_entry_runtime=0x1000003c090
creating offset page-table mapper
SMP: discovered=1 attempted=0 online=1 trampoline=false
active level-4 page table mapped
user mappings: code=0x400000 stack=0x7ff000 guard=0x7fe000 valid=true
mapping heap pages
heap pages mapped: 25
heap translation: true
user mapping permissions: true
process address space CR3 isolation: true
process teardown: first=true second=true
global allocator initialized
ELF loader PT_LOAD+BSS: true
memory: 62689 usable frames, 25 heap pages
paging + heap ready: Vec capacity=64, checksum=34464
scheduler smoke: completed=2, pending=0, runs=2
context switch smoke: true
VFS smoke: true, handles: true, ramdisk: true, global=true
syscall ABI smoke: true
IPC smoke: ports=true generation=true ownership=true
capability table: namespace=true opaque=true generation=true revoked=true
loopback: smoke=true checksum=true backpressure=true
loopback datagram: roundtrip=true ports=1000->2000 checksum=true corruption_rejected=true
block device: ata_pio_sector0=false mbr_partitions=false
persistent fs: novelfs_read=false write_rollback=false
user memory validation: true
ELF parser validation: true
init ELF from ramdisk: fs=true loaded=true bytes=347
process table: init registered=true current=true state=Running worker=2 state=Ready worker=3 state=Ready
exception table ready; starting timer bring-up
hardware IRQs online: 4 timer ticks
keyboard queue ready: no pending scancode
preemptive scheduler: task0=4 task1=4 switches=9 starvation=false
EXCEPTION: BREAKPOINT (#BP) handled
breakpoint exception works after hardware IRQs
user preemption: contexts=3 timer_frame=true
RING3 transition: init ELF iretq submitted
RING3 int80: cpl=3 valid=true
init from elf
SYSCALL write: accepted length=14
RING3 int80: cpl=3 valid=true
SYSCALL open: handle=1
RING3 int80: cpl=3 valid=true
SYSCALL read: handle=1 bytes=14
RING3 int80: cpl=3 valid=true
NOVEL_OS INIT
SYSCALL write: accepted length=14
RING3 int80: cpl=3 valid=true
SYSCALL close: handle=1 closed=true
RING3 int80: cpl=3 valid=true
SYSCALL port_create: handle=2
RING3 int80: cpl=3 valid=true
SYSCALL port_send: handle=2 bytes=3
RING3 int80: cpl=3 valid=true
SYSCALL port_recv: handle=2 bytes=3
RING3 int80: cpl=3 valid=true
init from elf
SYSCALL write: accepted length=14
RING3 int80: cpl=3 valid=true
SYSCALL open: handle=4194305
RING3 int80: cpl=3 valid=true
SYSCALL read: handle=4194305 bytes=14
RING3 int80: cpl=3 valid=true
NOVEL_OS INIT
SYSCALL write: accepted length=14
RING3 int80: cpl=3 valid=true
SYSCALL close: handle=4194305 closed=true
RING3 int80: cpl=3 valid=true
SYSCALL port_create: handle=257
RING3 int80: cpl=3 valid=true
SYSCALL port_send: handle=257 bytes=3
RING3 int80: cpl=3 valid=true
SYSCALL port_recv: handle=257 bytes=3
RING3 int80: cpl=3 valid=true
exit from elf
SYSCALL write: accepted length=14
RING3 int80: cpl=3 valid=true
SYSCALL exit: handoff status=0
user preemption: switches=2 frames=true starvation=false
PROCESS EXIT: status=0 owner=3 state=Exited revoked_handles=0 revoked_ports=0 reclaimed=true next=true
PROCESS HANDOFF: from=3 to=1 state=Running cr3=switched active=true resume=saved
USER FAULT POLICY: kill=true status=61444 address=Ok(VirtAddr(0x7fe000)) error=PageFaultErrorCode(USER_MODE)
user preemption: switches=3 frames=true starvation=false
PROCESS EXIT: status=61444 owner=2 state=Faulted revoked_handles=0 revoked_ports=1 reclaimed=true next=true
PROCESS HANDOFF: from=2 to=1 state=Running cr3=switched active=true resume=saved
USER FAULT POLICY: kill=true status=61444 address=Ok(VirtAddr(0x7fe000)) error=PageFaultErrorCode(USER_MODE)
user preemption: switches=3 frames=true starvation=false
PROCESS EXIT: status=61444 owner=1 state=Faulted revoked_handles=0 revoked_ports=1 reclaimed=true next=false
USER QUEUE: idle=true runnable=0
UEFI gate: PASS
artifacts=novel_os-bios.img,novel_os-uefi.img
bios_sha256=b6815ea30dc0cdd00f73c51a3402ef8838f30436aaa33984c38e99046434afc9
uefi_sha256=a5112e13b18f8f4273ebe8096125c6a76c22cd22453e9363f5a61a0768638a8b
cargo_lock_sha256=841b6bec89619a5d32719cfd684f41f00bf3afdbef6292605691ac017c100025
git_commit=5d79656f789e418013fd377a50870dbecdfeaf75
git_tag=smp-verification-evidence-v1-dirty
rustc=rustc 1.100.0-nightly (c656540d6 2026-08-21)
scope=BIOS and UEFI development kernel platform milestone; OVMF verified
excluded=SMP,NIC,DMA,general-persistent-filesystem,dynamic-run-queue,FPU-SIMD,production-userland,CI-external

## SMP low gate
ACPI: rsdp=true xsdt=false madt=true cpus=2 ioapics=1 lapic=0xfee00000 ioapic=0xfec00000
APIC routing: lapic=true ioapic=true timer_vector=32 keyboard_vector=33 local_id=0
SMP: discovered=2 attempted=1 online=2 trampoline=true
SYSCALL exit: handoff status=0
USER QUEUE: idle=true runnable=0
SMP low gate: PASS

## host tests

[1m[33mwarning[0m[1m: function `read_u64` is never used[0m
   [1m[94m--> [0mkernel/src/user.rs:201:4
    [1m[94m|[0m
[1m[94m201[0m [1m[94m|[0m fn read_u64(bytes: &[u8], offset: usize) -> Option<u64> {
    [1m[94m|[0m    [1m[33m^^^^^^^^[0m

[1m[33mwarning[0m[1m: function `elf_parser_smoke_test` is never used[0m
   [1m[94m--> [0mkernel/src/user.rs:207:8
    [1m[94m|[0m
[1m[94m207[0m [1m[94m|[0m pub fn elf_parser_smoke_test() -> bool {
    [1m[94m|[0m        [1m[33m^^^^^^^^^^^^^^^^^^^^^[0m

[1m[33mwarning[0m[1m: function `enter_user` is never used[0m
   [1m[94m--> [0mkernel/src/user.rs:214:15
    [1m[94m|[0m
[1m[94m214[0m [1m[94m|[0m pub unsafe fn enter_user(entry: VirtAddr, stack: VirtAddr, selectors: c[1m[94m...[0m
    [1m[94m|[0m               [1m[33m^^^^^^^^^^[0m

[1m[33mwarning[0m[1m: feature `alloc_error_handler` is declared but not used[0m
 [1m[94m--> [0mkernel/src/main.rs:4:12
  [1m[94m|[0m
[1m[94m4[0m [1m[94m|[0m #![feature(alloc_error_handler)]
  [1m[94m|[0m            [1m[33m^^^^^^^^^^^^^^^^^^^[0m
  [1m[94m|[0m
  [1m[94m= [0m[1mnote[0m: `#[warn(unused_features)]` (part of `#[warn(unused)]`) on by default

[1mFor more information about this error, try `rustc --explain E0133`.[0m
[1m[33mwarning[0m: `kernel` (bin "kernel" test) generated 229 warnings (run `cargo fix --bin "kernel" -p kernel --tests` to apply 3 suggestions)
[1m[92m    Finished[0m `test` profile [unoptimized + debuginfo] target(s) in 0.41s
[1m[92m     Running[0m unittests src/main.rs (target/debug/build/kernel/ed21cab9e4eeaacb/out/kernel-ed21cab9e4eeaacb)

running 4 tests
test acpi::tests::parses_enabled_cpu_and_ioapic ... ok
test acpi::tests::rejects_bad_rsdp_signature_and_checksum ... ok
test acpi::tests::accepts_valid_xsdt_rsdp ... ok
test acpi::tests::rejects_malformed_madt_entry ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

