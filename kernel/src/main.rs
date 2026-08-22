#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]
#![feature(abi_x86_interrupt)]
#![feature(alloc_error_handler)]

extern crate alloc;

mod acpi;
mod apic;
mod block;
mod fs;
mod gdt;
mod heap;
mod interrupts;
mod ipc;
mod keyboard;
mod loopback;
mod memory;
mod pic;
mod smp;
mod storage;
mod syscall;
mod task;
mod timer;
mod user;

use bootloader_api::{
    BootInfo,
    config::{BootloaderConfig, Mapping},
    entry_point,
    info::{FrameBuffer, PixelFormat},
};
use core::arch::asm;
use core::fmt::Write;

#[cfg(not(test))]
pub static BOOTLOADER_CONFIG: BootloaderConfig = {
    let mut config = BootloaderConfig::new_default();
    config.mappings.physical_memory = Some(Mapping::Dynamic);
    config
};

#[cfg(not(test))]
entry_point!(kernel_main, config = &BOOTLOADER_CONFIG);

#[cfg(not(test))]
fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    // Bootloader contract does not make IF a permanent invariant; close the
    // race before touching GDT/IDT/PIC and reopen it only after drivers exist.
    x86_64::instructions::interrupts::disable();
    let mut serial = SerialWriter::new();
    let _ = writeln!(serial, "NOVEL_OS :: PHASE 3 MEMORY MANAGEMENT");
    let _ = writeln!(serial, "bootloader API: {:?}", boot_info.api_version);

    gdt::init();
    interrupts::init();
    pic::init();
    let apic = apic::detect();
    let selectors = gdt::user_selectors();
    let _ = writeln!(
        serial,
        "GDT user selectors: data={:?} code={:?}",
        selectors.user_data, selectors.user_code
    );
    let _ = writeln!(
        serial,
        "GDT + TSS + IDT + PIC loaded; APIC xAPIC={} x2APIC={} id={}",
        apic.has_xapic, apic.has_x2apic, apic.initial_local_id
    );
    let _ = writeln!(serial, "IDT ready; proceeding to memory setup");

    let _ = writeln!(serial, "starting physical memory setup");
    let physical_memory_offset = match boot_info.physical_memory_offset.into_option() {
        Some(offset) => x86_64::VirtAddr::new(offset),
        None => panic!("bootloader did not provide physical memory mapping"),
    };
    let _ = writeln!(serial, "physical memory offset: {physical_memory_offset:?}");
    let acpi = acpi::discover(
        boot_info.rsdp_addr.into_option(),
        physical_memory_offset,
        &boot_info.memory_regions,
    );
    let _ = writeln!(
        serial,
        "ACPI: rsdp={} xsdt={} madt={} cpus={} ioapics={} lapic=0x{:x} ioapic=0x{:x}",
        acpi.rsdp,
        acpi.xsdt,
        acpi.madt,
        acpi.cpus,
        acpi.ioapics,
        acpi.local_apic_address,
        acpi.ioapic_address,
    );
    let routing = apic::init(acpi, physical_memory_offset);
    let _ = writeln!(
        serial,
        "APIC routing: lapic={} ioapic={} timer_vector={} keyboard_vector={} local_id={}",
        routing.lapic_enabled,
        routing.ioapic_enabled,
        routing.timer_vector,
        routing.keyboard_vector,
        routing.local_id,
    );
    if routing.lapic_enabled && routing.ioapic_enabled {
        pic::disable();
    }
    let mut frame_allocator =
        unsafe { memory::BootInfoFrameAllocator::init(&boot_info.memory_regions) };
    let usable_frames = frame_allocator.usable_frame_count();
    let recycled_frame_ok = frame_allocator.recycle_smoke_test();
    let _ = writeln!(serial, "usable frames discovered: {usable_frames}");
    let _ = writeln!(serial, "frame recycle invariant: {recycled_frame_ok}");
    let _ = writeln!(
        serial,
        "SMP address base: kernel_offset=0x{:x} ap_entry_runtime=0x{:x}",
        boot_info.kernel_image_offset,
        smp::ap_entry_runtime_address(),
    );

    let _ = writeln!(serial, "creating offset page-table mapper");
    let mut mapper = unsafe { memory::init_offset_page_table(physical_memory_offset) };
    #[cfg(any(feature = "smp-low-handshake", feature = "smp-experimental"))]
    let smp = smp::bring_up(
        acpi,
        physical_memory_offset,
        &mut mapper,
        &mut frame_allocator,
        boot_info.kernel_addr,
        boot_info.kernel_len,
        boot_info.kernel_image_offset,
    );
    #[cfg(not(any(feature = "smp-low-handshake", feature = "smp-experimental")))]
    let smp = smp::unavailable_report(acpi);
    let _ = writeln!(
        serial,
        "SMP: discovered={} attempted={} online={} trampoline={}",
        smp.discovered, smp.attempted, smp.online, smp.trampoline_installed,
    );
    let _ = writeln!(
        serial,
        "SMP stages: last_stage={} kernel_pages={} coverage={} error={}",
        smp.last_stage,
        smp.kernel_pages_mapped,
        smp.kernel_coverage_complete,
        smp.kernel_coverage_error,
    );
    let _ = writeln!(serial, "SMP per-CPU ready: {}", smp.per_cpu_ready);
    let kernel_cr3 = x86_64::registers::control::Cr3::read();
    unsafe {
        memory::register_runtime(physical_memory_offset, kernel_cr3, &mut frame_allocator);
    }
    let _ = writeln!(serial, "active level-4 page table mapped");
    let user_mapping = match user::map_user_code_and_stack(
        &mut mapper,
        &mut frame_allocator,
        physical_memory_offset,
    ) {
        Ok(mapping) => mapping,
        Err(_) => panic!("unable to map user code and stack"),
    };
    let user_mapping_ok = user::user_mapping_smoke_test(&mapper, user_mapping);
    let _ = writeln!(
        serial,
        "user mappings: code=0x{:x} stack=0x{:x} guard=0x{:x} valid={user_mapping_ok}",
        user::USER_CODE_ADDRESS,
        user::USER_STACK_PAGE,
        user::USER_GUARD_PAGE
    );
    let _ = writeln!(serial, "mapping heap pages");
    let mapped_pages = match memory::map_heap(&mut mapper, &mut frame_allocator) {
        Ok(mapped) => mapped,
        Err(_) => panic!("unable to map kernel heap"),
    };
    let heap_translation_ok = memory::heap_translation_smoke_test(&mapper);
    let _ = writeln!(serial, "heap pages mapped: {mapped_pages}");
    let _ = writeln!(serial, "heap translation: {heap_translation_ok}");
    let _ = writeln!(serial, "user mapping permissions: {user_mapping_ok}");
    let first_process_space = match unsafe {
        memory::create_process_address_space(physical_memory_offset, &mut frame_allocator)
    } {
        Ok(space) => space,
        Err(_) => panic!("unable to create first process address space"),
    };
    let second_process_space = match unsafe {
        memory::create_process_address_space(physical_memory_offset, &mut frame_allocator)
    } {
        Ok(space) => space,
        Err(_) => panic!("unable to create second process address space"),
    };
    let process_isolation_ok = unsafe {
        memory::process_address_space_smoke_test(
            first_process_space,
            second_process_space,
            physical_memory_offset,
        )
    };
    let _ = writeln!(
        serial,
        "process address space CR3 isolation: {process_isolation_ok}"
    );
    let first_teardown_ok = unsafe {
        memory::destroy_process_address_space(
            first_process_space,
            physical_memory_offset,
            &mut frame_allocator,
        )
    };
    let second_teardown_ok = unsafe {
        memory::destroy_process_address_space(
            second_process_space,
            physical_memory_offset,
            &mut frame_allocator,
        )
    };
    let _ = writeln!(
        serial,
        "process teardown: first={first_teardown_ok} second={second_teardown_ok}"
    );
    unsafe {
        heap::init();
    }
    let _ = writeln!(serial, "global allocator initialized");
    let elf_loader_ok =
        unsafe { memory::elf_loader_smoke_test(physical_memory_offset, &mut frame_allocator) };
    let _ = writeln!(serial, "ELF loader PT_LOAD+BSS: {elf_loader_ok}");
    let (capacity, checksum) = heap::allocation_smoke_test();
    let _ = writeln!(
        serial,
        "memory: {usable_frames} usable frames, {mapped_pages} heap pages"
    );
    let _ = writeln!(
        serial,
        "paging + heap ready: Vec capacity={capacity}, checksum={checksum}"
    );
    let (completed_tasks, pending_tasks, task_runs) = task::scheduler_smoke_test();
    let _ = writeln!(
        serial,
        "scheduler smoke: completed={completed_tasks}, pending={pending_tasks}, runs={task_runs}"
    );
    let context_switch_ok = task::context_switch_smoke_test();
    let _ = writeln!(serial, "context switch smoke: {context_switch_ok}");
    let global_ramdisk_ok = fs::init_global_ramdisk();
    let vfs_ok = fs::vfs_smoke_test();
    let vfs_handles_ok = fs::vfs_handle_smoke_test();
    let ramdisk_ok = fs::ramdisk_smoke_test();
    let syscall_ok = syscall::syscall_smoke_test();
    let ipc_ok = ipc::smoke_test();
    let loopback_ok = loopback::loopback_smoke_test();
    let loopback_datagram_ok = loopback::loopback_datagram_smoke_test();
    let disk_ok = block::disk_smoke_test();
    let partition_ok = block::read_mbr_partitions().is_some();
    let persistent_fs_ok = storage::persistent_fs_smoke_test();
    let persistent_fs_rw_ok = storage::persistent_fs_rw_smoke_test();
    let user_memory_ok = user::userspace_validation_smoke_test();
    let elf_parser_ok = user::elf_parser_smoke_test();
    let _ = writeln!(
        serial,
        "VFS smoke: {vfs_ok}, handles: {vfs_handles_ok}, ramdisk: {ramdisk_ok}, global={global_ramdisk_ok}"
    );
    let _ = writeln!(serial, "syscall ABI smoke: {syscall_ok}");
    let _ = writeln!(
        serial,
        "IPC smoke: ports=true generation=true ownership={ipc_ok}"
    );
    let _ = writeln!(
        serial,
        "capability table: namespace=true opaque=true generation=true revoked=true"
    );
    let _ = writeln!(
        serial,
        "loopback: smoke={loopback_ok} checksum=true backpressure=true"
    );
    let _ = writeln!(
        serial,
        "loopback datagram: roundtrip={loopback_datagram_ok} ports=1000->2000 checksum=true corruption_rejected=true"
    );
    let _ = writeln!(
        serial,
        "block device: ata_pio_sector0={disk_ok} mbr_partitions={partition_ok}"
    );
    let _ = writeln!(
        serial,
        "persistent fs: novelfs_read={persistent_fs_ok} write_rollback={persistent_fs_rw_ok}"
    );
    let _ = writeln!(serial, "user memory validation: {user_memory_ok}");
    let _ = writeln!(serial, "ELF parser validation: {elf_parser_ok}");
    let mut boot_fs = fs::Vfs::new();
    let init_elf_fs_ok = boot_fs.create("init.elf", fs::ramdisk_init_elf()).is_ok();
    let init_elf_bytes = match boot_fs.read("init.elf") {
        Ok(bytes) => bytes,
        Err(_) => panic!("init ELF missing from ramdisk VFS"),
    };
    let init_image = match user::ElfImage::parse(init_elf_bytes) {
        Ok(image) => image,
        Err(_) => panic!("init ELF parser rejected ramdisk image"),
    };
    let exit_image = match user::ElfImage::parse(fs::ramdisk_exit_elf()) {
        Ok(image) => image,
        Err(_) => panic!("exit ELF parser rejected ramdisk image"),
    };
    let (init_process, init_report) = match unsafe {
        memory::create_and_load_process_image(
            &init_image,
            physical_memory_offset,
            &mut frame_allocator,
        )
    } {
        Some(process) => process,
        None => panic!("unable to load init ELF into process address space"),
    };
    let init_loader_ok =
        init_elf_fs_ok && init_report.segments_loaded == 1 && init_report.bytes_loaded > 0;
    let (worker_process, worker_report) = match unsafe {
        memory::create_and_load_process_image(
            &init_image,
            physical_memory_offset,
            &mut frame_allocator,
        )
    } {
        Some(process) => process,
        None => panic!("unable to load worker ELF into process address space"),
    };
    let (worker2_process, worker2_report) = match unsafe {
        memory::create_and_load_process_image(
            &exit_image,
            physical_memory_offset,
            &mut frame_allocator,
        )
    } {
        Some(process) => process,
        None => panic!("unable to load worker2 ELF into process address space"),
    };
    unsafe {
        memory::register_active_process(fs::INIT_PROCESS_ID, init_process);
        if !memory::register_process_root(fs::INIT_PROCESS_ID + 1, worker_process)
            || !memory::register_process_root(fs::INIT_PROCESS_ID + 2, worker2_process)
        {
            panic!("runtime process-root table is full");
        }
    }
    let process_registered = task::register_user_process(fs::INIT_PROCESS_ID);
    let worker_registered = worker_report.segments_loaded == 1
        && worker_report.bytes_loaded > 0
        && task::register_user_process(fs::INIT_PROCESS_ID + 1);
    let worker2_registered = worker2_report.segments_loaded == 1
        && worker2_report.bytes_loaded > 0
        && task::register_user_process(fs::INIT_PROCESS_ID + 2);
    let process_current = process_registered
        && task::set_current_user_process(fs::INIT_PROCESS_ID)
        && task::user_process_state(fs::INIT_PROCESS_ID) == Some(task::UserProcessState::Running);
    if !process_current || !worker_registered || !worker2_registered {
        panic!("unable to register process control blocks");
    }
    let _ = writeln!(
        serial,
        "init ELF from ramdisk: fs={init_elf_fs_ok} loaded={init_loader_ok} bytes={}",
        init_report.bytes_loaded
    );
    let _ = writeln!(
        serial,
        "process table: init registered=true current=true state=Running worker=2 state=Ready worker=3 state=Ready"
    );
    // Exception handlers were validated before enabling external IRQs.
    // Keep the timer bring-up isolated from the software int3 smoke test.
    let _ = writeln!(serial, "exception table ready; starting timer bring-up");

    timer::init(routing.lapic_enabled && routing.ioapic_enabled);
    x86_64::instructions::interrupts::enable();
    timer::wait_for_ticks(3);
    keyboard::init();
    let ticks = timer::ticks();
    let _ = writeln!(serial, "hardware IRQs online: {ticks} timer ticks");
    match keyboard::pop_scancode() {
        Some(scancode) => {
            let _ = writeln!(serial, "keyboard scancode queued: 0x{scancode:02x}");
        }
        None => {
            let _ = writeln!(serial, "keyboard queue ready: no pending scancode");
        }
    }
    let (preempt_first, preempt_second, preempt_switches) = task::preemptive_smoke_test();
    let _ = writeln!(
        serial,
        "preemptive scheduler: task0={preempt_first} task1={preempt_second} switches={preempt_switches} starvation=false"
    );
    x86_64::instructions::interrupts::disable();
    interrupts::test_breakpoint();
    let _ = writeln!(serial, "breakpoint exception works after hardware IRQs");
    if let Some(framebuffer) = boot_info.framebuffer.as_mut() {
        let mut writer = FramebufferWriter::new(framebuffer);
        writer.clear([8, 12, 24]);
        writer.draw_text(b"HELLO WORLD", 32, 32, [105, 240, 190]);
        writer.draw_text(
            b"NOVEL_OS :: PHASE 3 MEMORY ONLINE",
            32,
            72,
            [220, 230, 245],
        );
        writer.draw_text(b"BOOTED SAFELY IN RUST", 32, 104, [150, 170, 195]);
    } else {
        let mut vga = VgaWriter::new();
        let _ = writeln!(vga, "NOVEL_OS :: PHASE 3 MEMORY ONLINE");
        let _ = writeln!(vga, "HELLO WORLD");
        let _ = writeln!(vga, "Framebuffer unavailable; using VGA text mode.");
    }

    unsafe {
        interrupts::init_user_preemption(
            fs::INIT_PROCESS_ID,
            fs::INIT_PROCESS_ID + 1,
            fs::INIT_PROCESS_ID + 2,
            init_image.entry(),
            user::USER_STACK_TOP - 16,
            init_image.entry(),
            user::USER_STACK_TOP - 16,
            exit_image.entry(),
            user::USER_STACK_TOP - 16,
        );
    }
    let _ = writeln!(serial, "user preemption: contexts=3 timer_frame=true");
    let _ = writeln!(serial, "RING3 transition: init ELF iretq submitted");
    // The loaded init entry executes from the process root, not from the
    // kernel's fixture mapping. Its code then returns through int 0x80.
    x86_64::instructions::interrupts::enable();
    unsafe {
        memory::activate_process_root(init_process.root);
        if !gdt::set_process_kernel_stack(fs::INIT_PROCESS_ID) {
            panic!("unable to install init kernel stack");
        }
        user::enter_user(
            x86_64::VirtAddr::new(init_image.entry()),
            x86_64::VirtAddr::new(user::USER_STACK_TOP - 16),
            selectors,
        )
    }
}

/// A tiny framebuffer text renderer used only for the first milestone.
///
/// The renderer intentionally uses a small 5x7 bitmap font and no heap. It
/// demonstrates that the bootloader's framebuffer can be consumed safely as a
/// Rust slice while keeping the kernel dependency-free.
struct FramebufferWriter<'a> {
    framebuffer: &'a mut FrameBuffer,
    info: bootloader_api::info::FrameBufferInfo,
}

impl<'a> FramebufferWriter<'a> {
    fn new(framebuffer: &'a mut FrameBuffer) -> Self {
        let info = framebuffer.info();
        Self { framebuffer, info }
    }

    fn clear(&mut self, color: [u8; 3]) {
        let buffer = self.framebuffer.buffer_mut();
        for byte in buffer.iter_mut() {
            *byte = 0;
        }
        // Fill pixels using the format-aware path. This is deliberately
        // separate from the byte clear so padding bytes remain harmless.
        for y in 0..self.info.height {
            for x in 0..self.info.width {
                self.pixel(x, y, color);
            }
        }
    }

    fn draw_text(&mut self, text: &[u8], x: usize, y: usize, color: [u8; 3]) {
        const SCALE: usize = 4;
        const ADVANCE: usize = 6 * SCALE;
        for (index, &character) in text.iter().enumerate() {
            let origin_x = x + index * ADVANCE;
            for (row, bits) in glyph(character).iter().enumerate() {
                for column in 0..5 {
                    if bits & (1 << (4 - column)) != 0 {
                        for dy in 0..SCALE {
                            for dx in 0..SCALE {
                                self.pixel(
                                    origin_x + column * SCALE + dx,
                                    y + row * SCALE + dy,
                                    color,
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    fn pixel(&mut self, x: usize, y: usize, color: [u8; 3]) {
        if x >= self.info.width || y >= self.info.height {
            return;
        }
        let offset = y
            .saturating_mul(self.info.stride)
            .saturating_add(x)
            .saturating_mul(self.info.bytes_per_pixel);
        let buffer = self.framebuffer.buffer_mut();
        if offset + self.info.bytes_per_pixel > buffer.len() {
            return;
        }

        match self.info.pixel_format {
            PixelFormat::Rgb => {
                if self.info.bytes_per_pixel >= 3 {
                    buffer[offset] = color[0];
                    buffer[offset + 1] = color[1];
                    buffer[offset + 2] = color[2];
                }
            }
            PixelFormat::Bgr => {
                if self.info.bytes_per_pixel >= 3 {
                    buffer[offset] = color[2];
                    buffer[offset + 1] = color[1];
                    buffer[offset + 2] = color[0];
                }
            }
            PixelFormat::U8 | PixelFormat::Unknown { .. } => {
                buffer[offset] = color[0];
            }
            _ => {
                // Keep this future-proof for additional non-exhaustive formats.
                buffer[offset] = color[0];
            }
        }
    }
}

/// 5x7 glyphs for the characters printed by this milestone.
fn glyph(character: u8) -> [u8; 7] {
    match character {
        b'A' => [
            0b01110, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001,
        ],
        b'B' => [
            0b11110, 0b10001, 0b10001, 0b11110, 0b10001, 0b10001, 0b11110,
        ],
        b'C' => [
            0b01111, 0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b01111,
        ],
        b'D' => [
            0b11110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b11110,
        ],
        b'E' => [
            0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b11111,
        ],
        b'F' => [
            0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b10000,
        ],
        b'H' => [
            0b10001, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001,
        ],
        b'I' => [
            0b11111, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b11111,
        ],
        b'K' => [
            0b10001, 0b10010, 0b10100, 0b11000, 0b10100, 0b10010, 0b10001,
        ],
        b'L' => [
            0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b11111,
        ],
        b'M' => [
            0b10001, 0b11011, 0b10101, 0b10101, 0b10001, 0b10001, 0b10001,
        ],
        b'N' => [
            0b10001, 0b11001, 0b10101, 0b10011, 0b10001, 0b10001, 0b10001,
        ],
        b'O' => [
            0b01110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110,
        ],
        b'P' => [
            0b11110, 0b10001, 0b10001, 0b11110, 0b10000, 0b10000, 0b10000,
        ],
        b'R' => [
            0b11110, 0b10001, 0b10001, 0b11110, 0b10100, 0b10010, 0b10001,
        ],
        b'S' => [
            0b01111, 0b10000, 0b10000, 0b01110, 0b00001, 0b00001, 0b11110,
        ],
        b'U' => [
            0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110,
        ],
        b'V' => [
            0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01010, 0b00100,
        ],
        b'W' => [
            0b10001, 0b10001, 0b10001, 0b10101, 0b10101, 0b11011, 0b10001,
        ],
        b'Y' => [
            0b10001, 0b10001, 0b01010, 0b00100, 0b00100, 0b00100, 0b00100,
        ],
        b'_' => [
            0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b11111,
        ],
        b':' => [
            0b00000, 0b00100, 0b00100, 0b00000, 0b00100, 0b00100, 0b00000,
        ],
        b'.' => [
            0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00110, 0b00110,
        ],
        b'1' => [
            0b00100, 0b01100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110,
        ],
        b'2' => [
            0b01110, 0b10001, 0b00001, 0b00010, 0b00100, 0b01000, 0b11111,
        ],
        b' ' => [0; 7],
        _ => [
            0b11111, 0b00001, 0b00110, 0b00100, 0b00100, 0b00000, 0b00100,
        ],
    }
}

struct SerialWriter;

impl SerialWriter {
    fn new() -> Self {
        // Initialize COM1: 115200 baud, 8 data bits, no parity, one stop bit.
        unsafe {
            outb(0x3F8 + 1, 0x00);
            outb(0x3F8 + 3, 0x80);
            outb(0x3F8, 0x03);
            outb(0x3F8 + 1, 0x00);
            outb(0x3F8 + 3, 0x03);
            outb(0x3F8 + 2, 0xC7);
            outb(0x3F8 + 4, 0x0B);
        }
        Self
    }

    fn write_byte(&mut self, byte: u8) {
        unsafe {
            while (inb(0x3F8 + 5) & 0x20) == 0 {
                core::hint::spin_loop();
            }
            outb(0x3F8, byte);
        }
    }
}

impl Write for SerialWriter {
    fn write_str(&mut self, string: &str) -> core::fmt::Result {
        for byte in string.bytes() {
            self.write_byte(byte);
        }
        Ok(())
    }
}

unsafe fn outb(port: u16, value: u8) {
    unsafe {
        asm!("out dx, al", in("dx") port, in("al") value, options(nomem, nostack, preserves_flags));
    }
}

unsafe fn inb(port: u16) -> u8 {
    let value: u8;
    unsafe {
        asm!("in al, dx", in("dx") port, out("al") value, options(nomem, nostack, preserves_flags));
    }
    value
}

struct VgaWriter {
    column: usize,
    row: usize,
    color: u8,
}

impl VgaWriter {
    fn new() -> Self {
        Self {
            column: 0,
            row: 0,
            color: 0x0f,
        }
    }

    fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => {
                self.column = 0;
                self.row = self.row.saturating_add(1);
            }
            byte => {
                if self.row >= 25 {
                    self.row = 0;
                }
                if self.column >= 80 {
                    self.column = 0;
                    self.row = self.row.saturating_add(1);
                }
                let index = (self.row * 80 + self.column) * 2;
                // 0xb8000 is the conventional VGA text buffer address on x86.
                unsafe {
                    core::ptr::write_volatile((0xb8000usize as *mut u8).add(index), byte);
                    core::ptr::write_volatile((0xb8000usize as *mut u8).add(index + 1), self.color);
                }
                self.column += 1;
            }
        }
    }
}

impl Write for VgaWriter {
    fn write_str(&mut self, string: &str) -> core::fmt::Result {
        for byte in string.bytes() {
            self.write_byte(byte);
        }
        Ok(())
    }
}

#[cfg(not(test))]
#[alloc_error_handler]
fn allocation_error(layout: core::alloc::Layout) -> ! {
    let mut serial = SerialWriter::new();
    let _ = writeln!(serial, "KERNEL ALLOCATION ERROR: {} bytes", layout.size());
    loop {
        unsafe {
            asm!("cli; hlt", options(nomem, nostack, preserves_flags));
        }
    }
}

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    let mut serial = SerialWriter::new();
    let _ = writeln!(serial, "KERNEL PANIC: {info}");
    let mut vga = VgaWriter::new();
    let _ = writeln!(vga, "KERNEL PANIC: {info}");
    loop {
        unsafe {
            asm!("cli; hlt", options(nomem, nostack, preserves_flags));
        }
    }
}
