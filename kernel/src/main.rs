#![no_std]
#![no_main]

use core::arch::asm;
use core::str::from_utf8;
extern crate alloc;
use alloc::vec;
use catmeow_os::filesystem::init_filesystem;
use catmeow_os::gdt::init_gdt;
use catmeow_os::gui::init_screen;
use catmeow_os::idt::{enable_interrupts, init_idt};
use catmeow_os::memory::vmm::{allocate, allocate_with_ptr, unmap_page};
use catmeow_os::memory::{PAGE_SIZE, heap, vmm};
use catmeow_os::memory::{
    PhysicalPointer,
    pmm::{self, get_hhdm_offset, init_memory, page_align},
};
use catmeow_os::pit::{get_ms, init_pit, remap_pic};
use catmeow_os::scheduler::init_scheduler;
use catmeow_os::serial::init_serial;
#[cfg(target_arch = "x86_64")]
use catmeow_os::serial_print;
use catmeow_os::terminal::init_terminal;
use catmeow_os::tsc::init_tsc;
use catmeow_os::{acpi, serial_println, terminal_println};
use limine::BaseRevision;
use limine::modules::{InternalModule, ModuleFlags};
use limine::paging::Mode;
use limine::request::{
    EntryPointRequest, ExecutableAddressRequest, FramebufferRequest, HhdmRequest, MemoryMapRequest,
    ModuleRequest, MpRequest, PagingModeRequest, RequestsEndMarker, RequestsStartMarker,
    RsdpRequest, SmpRequest,
};
/// Sets the base revision to the latest revision supported by the crate.
/// See specification for further info.
/// Be sure to mark all limine requests with #[used], otherwise they may be removed by the compiler.
#[used]
// The .requests section allows limine to find the requests faster and more safely.
#[unsafe(link_section = ".requests")]
static BASE_REVISION: BaseRevision = BaseRevision::new();

#[used]
#[unsafe(link_section = ".requests")]
static FRAMEBUFFER_REQUEST: FramebufferRequest = FramebufferRequest::new();

#[used]
#[unsafe(link_section = ".requests")]
static MEMORY_MAP_REQUEST: MemoryMapRequest = MemoryMapRequest::new();

#[used]
#[unsafe(link_section = ".requests")]
static PAGING_MODE_REQUEST: PagingModeRequest = PagingModeRequest::new();

#[used]
#[unsafe(link_section = ".requests")]
static EXECUTABLE_ADDRESS_REQUEST: ExecutableAddressRequest = ExecutableAddressRequest::new();

#[used]
#[unsafe(link_section = ".requests")]
static RSDP_REQUEST: RsdpRequest = RsdpRequest::new();

#[used]
#[unsafe(link_section = ".requests")]
static MP_REQUEST: MpRequest = MpRequest::new();

#[used]
#[unsafe(link_section = ".requests")]
static MODULE_REQUEST: ModuleRequest =
    ModuleRequest::new().with_internal_modules(&[&RAMDISK_MODULE]);
const RAMDISK_MODULE: InternalModule = InternalModule::new()
    .with_path(c"/ramdisk.tar")
    .with_flags(ModuleFlags::REQUIRED);

#[used]
#[unsafe(link_section = ".requests")]
static HHDM_REQUEST: HhdmRequest = HhdmRequest::new();

/// Define the stand and end markers for Limine requests.
#[used]
#[unsafe(link_section = ".requests_start_marker")]
static _START_MARKER: RequestsStartMarker = RequestsStartMarker::new();
#[used]
#[unsafe(link_section = ".requests_end_marker")]
static _END_MARKER: RequestsEndMarker = RequestsEndMarker::new();

#[unsafe(no_mangle)]
unsafe extern "C" fn kmain() -> ! {
    // All limine requests must also be referenced in a called function, otherwise they may be
    // removed by the linker.
    assert!(BASE_REVISION.is_supported());
    init_serial();
    if let Some(framebuffer_response) = FRAMEBUFFER_REQUEST.get_response() {
        if let Some(framebuffer) = framebuffer_response.framebuffers().next() {
            for i in 0..100_u64 {
                // Calculate the pixel offset using the framebuffer information we obtained above.
                // We skip `i` scanlines (pitch is provided in bytes) and add `i * 4` to skip `i` pixels forward.
                let pixel_offset = i * framebuffer.pitch() + i * 4;

                // Write 0xFFFFFFFF to the provided pixel offset to fill it white.
                unsafe {
                    framebuffer
                        .addr()
                        .add(pixel_offset as usize)
                        .cast::<u32>()
                        .write(0xFFFFFFFF)
                };
            }

            init_terminal(&framebuffer);
            init_screen(&framebuffer);
        }
    }
    terminal_println!("start of os");
    init_gdt();
    terminal_println!("GDT INITIALIZED");
    init_idt();
    terminal_println!("IDT INITIALIZED");
    init_pit();
    terminal_println!("PIT INITIALIZED");
    init_tsc();
    terminal_println!("TSC INITIALIZED");
    if let Some(memory_map_response) = MEMORY_MAP_REQUEST.get_response() {
        if let Some(hhdm_response) = HHDM_REQUEST.get_response() {
            let offset = hhdm_response.offset();
            let entries = memory_map_response.entries();
            init_memory(entries, offset);
            terminal_println!("PMM INITIALIZED");
        }
    }
    if let Some(paging_mode_response) = PAGING_MODE_REQUEST.get_response() {
        let mode = paging_mode_response.mode();
        if mode == Mode::DEFAULT {
            terminal_println!("default paging");
        }
        if mode == Mode::FIVE_LEVEL {
            terminal_println!("five level paging");
        }
        if mode == Mode::FOUR_LEVEL {
            terminal_println!("four level paging");
        }
        if mode == Mode::MIN {
            terminal_println!("min paging paging")
        }
    }
    if let Some(executable_address_response) = EXECUTABLE_ADDRESS_REQUEST.get_response() {
        let physical_base = executable_address_response.physical_base();
        let virtual_base = executable_address_response.virtual_base();
        vmm::init_vmm(physical_base, virtual_base);
        terminal_println!("VMM INITIALIZED");
    }

    if let Some(rsdp_response) = RSDP_REQUEST.get_response() {
        let rsdp_address = rsdp_response.address();
        //acpi::init_acpi(rsdp_address);
    }
    if let Some(mp_response) = MP_REQUEST.get_response() {
        terminal_println!("cpu len: {:?}", mp_response.cpus().len());
        for cpu in mp_response.cpus() {
            terminal_println!("cpu: {:?}", cpu.id);
            terminal_println!("cpu lapic id: {:?}", cpu.lapic_id);
        }
    }

    heap::init_heap();
    terminal_println!("KHEAP INITIALIZED");
    let mut v = vec![1, 2, 3];
    v.push(67);
    v.push(41);
    v.push(255);
    for i in v {
        terminal_println!("{}", i);
    }
    init_filesystem(get_ramdisk_file());
    terminal_println!("FILESYSTEM INITIALIZED");
    init_scheduler();
    terminal_println!("SCHEDULER INITIALIZED");

    terminal_println!("end");
    enable_interrupts();
    hcf();
}

#[cfg(not(test))]
#[panic_handler]
fn rust_panic(info: &core::panic::PanicInfo) -> ! {
    terminal_println!("pc cooked :skull:");
    serial_println!("{}", info);
    terminal_println!("{}", info);
    hcf();
}

fn hcf() -> ! {
    loop {
        unsafe {
            #[cfg(target_arch = "x86_64")]
            asm!("hlt");
            #[cfg(any(target_arch = "aarch64", target_arch = "riscv64"))]
            asm!("wfi");
            #[cfg(target_arch = "loongarch64")]
            asm!("idle 0");
        }
    }
}
pub fn get_ramdisk_file() -> &'static limine::file::File {
    MODULE_REQUEST
        .get_response()
        .expect("failed getting modules!")
        .modules()[0]
}
