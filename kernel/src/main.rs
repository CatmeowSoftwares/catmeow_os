#![no_std]
#![no_main]

use catmeow_os::alloc::string::{String, ToString};
use catmeow_os::filesystem::Directory;
use catmeow_os::idt::init_idt;
use catmeow_os::pmm::BootInfoFrameAllocator;
use catmeow_os::scheduler::{Process, Scheduler};
use catmeow_os::serial::init_serial;
use catmeow_os::terminal::init_terminal;
use catmeow_os::vmm::{alloc_page, dealloc_page, init_heap};
use catmeow_os::{gdt::init_gdt, serial_println};
use catmeow_os::{pmm, terminal_print, terminal_println};
use core::arch::asm;
use core::ffi::CStr;
use core::ptr::{null, null_mut, slice_from_raw_parts, slice_from_raw_parts_mut};
use limine::BaseRevision;
use limine::file::File;
use limine::memory_map::Entry;
use limine::modules::{InternalModule, ModuleFlags};
use limine::request::{
    FramebufferRequest, HhdmRequest, MemoryMapRequest, ModuleRequest, RequestsEndMarker,
    RequestsStartMarker,
};
use spin::Mutex;
use x86_64::VirtAddr;
use x86_64::instructions::interrupts::int3;
use x86_64::structures::paging::OffsetPageTable;
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
static HHDM_REQUEST: HhdmRequest = HhdmRequest::new();

#[used]
#[unsafe(link_section = ".requests")]
static MEMORY_MAP_REQUEST: MemoryMapRequest = MemoryMapRequest::new();

const RAMDISK_MODULE: InternalModule = InternalModule::new()
    .with_path(c"/ramdisk.tar")
    .with_flags(ModuleFlags::REQUIRED);

#[used]
#[unsafe(link_section = ".requests")]
static MODULES_REQUEST: ModuleRequest =
    ModuleRequest::new().with_internal_modules(&[&RAMDISK_MODULE]);

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
                framebuffer.addr();
                // Write 0xFFFFFFFF to the provided pixel offset to fill it white.
                unsafe {
                    framebuffer
                        .addr()
                        .add(pixel_offset as usize)
                        .cast::<u32>()
                        .write(0xFFFFFFFF)
                };
            }
            init_terminal(framebuffer);
        }
    }
    init_gdt();
    init_idt();

    if let Some(hhdm_response) = HHDM_REQUEST.get_response() {
        let virt_addr = VirtAddr::new(hhdm_response.offset());

        let mut mapper: OffsetPageTable<'static> = unsafe { pmm::init(virt_addr) };

        if let Some(memory_map_response) = MEMORY_MAP_REQUEST.get_response() {
            let entries = memory_map_response.entries();
            let mut frame_allocator: BootInfoFrameAllocator =
                unsafe { BootInfoFrameAllocator::init(entries) };

            let _ = init_heap(&mut mapper, &mut frame_allocator);

            alloc_page(0xdeadbeef, 4096, &mut frame_allocator, &mut mapper).unwrap();
            let ptr = 0xdeadbeef as *mut u8;
            unsafe {
                //dealloc_page(0xdeadbeef, 4096,&mut mapper);
                let deref_ptr = *ptr;
                terminal_println!("{}", deref_ptr);
            }
        }
    }
    extern crate alloc;
    use alloc::{boxed::Box, rc::Rc, vec, vec::Vec};

    let file = get_ramdisk_file();
    let sliz = &*slice_from_raw_parts_mut(file.addr(), file.size() as usize);
    let archive = tar_no_std::TarArchiveRef::new(sliz).unwrap();
    let entries = archive.entries().collect::<Vec<_>>();

    terminal_println!();
    serial_println!("meow :3");
    serial_println!("filesystem");

    let mut root = Directory::new("root");
    for entry in entries {
        let file = catmeow_os::filesystem::File::new(entry.filename().as_str().unwrap())
            .with_data(entry.data());
        root.move_file(file);
        terminal_println!("{:#?}", entry);
    }

    let processes: Vec<Process> = Vec::new();

    let scheduler: Scheduler = Scheduler::new();

    for file in root.files() {
        if file.name == "root/print".to_string() {
            let data = file.data();
            let a = &data[0..4];
            let c = String::from_utf8(a.into()).unwrap();
            terminal_println!("{:?}", c);
        }
        if file.name == "root/hello_world.bin".to_string() {}
    }
    hcf();
}
#[cfg(not(test))]
#[panic_handler]
fn rust_panic(info: &core::panic::PanicInfo) -> ! {
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

fn set_pixel(x: u32, y: u32, color: u32) {}

pub fn get_ramdisk_file() -> &'static File {
    MODULES_REQUEST
        .get_response()
        .expect("failed getting modules!")
        .modules()[0]
}
/*
pub fn get_ramdisk() -> TarArc<'static> {
    unsafe { TarArchiveIter::new(get_ramdisk_file().addr()) }
}
 */
