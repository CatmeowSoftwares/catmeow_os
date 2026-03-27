#![no_std]
#![no_main]

use core::arch::asm;

use catmeow_os::gdt::init_gdt;
use catmeow_os::idt::init_idt;
use catmeow_os::memory::{self, init_memory};
use catmeow_os::serial::init_serial;
use catmeow_os::serial_println;
use limine::BaseRevision;
use limine::memory_map::EntryType;
use limine::paging::Mode;
use limine::request::{
    FramebufferRequest, MemoryMapRequest, PagingModeRequest, RequestsEndMarker, RequestsStartMarker,
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
    serial_println!("start of os");
    init_gdt();
    init_idt();
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
        }
        if let Some(memory_map_response) = MEMORY_MAP_REQUEST.get_response() {
            let entries = memory_map_response.entries();
            init_memory(entries);
        }
        if let Some(paging_mode_response) = PAGING_MODE_REQUEST.get_response() {
            let mode = paging_mode_response.mode();
            if mode == Mode::DEFAULT {
                serial_println!("default paging");
            }
            if mode == Mode::FIVE_LEVEL {
                serial_println!("five level paging");
            }
            if mode == Mode::FOUR_LEVEL {
                serial_println!("four level paging");
            }
            if mode == Mode::MIN {
                serial_println!("min paging paging")
            }
        }
    }
    serial_println!("end");
    hcf();
}

#[cfg(not(test))]
#[panic_handler]
fn rust_panic(info: &core::panic::PanicInfo) -> ! {
    serial_println!("{}", info);
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
