#![no_std]
#![no_main]

use core::arch::asm;
use core::ffi::{CStr, c_char, c_void};
use core::ptr::null;

use limine::BaseRevision;
use limine::request::{FramebufferRequest, RequestsEndMarker, RequestsStartMarker};

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

    if let Some(framebuffer_response) = FRAMEBUFFER_REQUEST.get_response() {
        if let Some(framebuffer) = framebuffer_response.framebuffers().next() {
            let a;
            unsafe {
                /*
                flanterm::sys::flanterm_fb_init(
                    malloc,
                    free,
                    framebuffer,
                    width,
                    height,
                    pitch,
                    red_mask_size,
                    red_mask_shift,
                    green_mask_size,
                    green_mask_shift,
                    blue_mask_size,
                    blue_mask_shift,
                    canvas,
                    ansi_colours,
                    ansi_bright_colours,
                    default_bg,
                    default_fg,
                    default_bg_bright,
                    default_fg_bright,
                    font,
                    font_width,
                    font_height,
                    font_spacing,
                    font_scale_x,
                    font_scale_y,
                    margin,
                );
                 */

                a = flanterm::sys::flanterm_fb_init(
                    None,
                    None,
                    framebuffer.addr() as *mut u32,
                    framebuffer.width() as usize,
                    framebuffer.height() as usize,
                    100,
                    100,
                    100,
                    100,
                    100,
                    100,
                    100,
                    0 as *mut u32,
                    0 as *mut u32,
                    0 as *mut u32,
                    0 as *mut u32,
                    0 as *mut u32,
                    0 as *mut u32,
                    0 as *mut u32,
                    0 as *mut c_void,
                    0,
                    0,
                    1,
                    0,
                    0,
                    0,
                );
                let msg = CStr::from_bytes_with_nul_unchecked(b"meow :3");
                flanterm::sys::flanterm_write(a, msg.as_ptr(), msg.count_bytes());
            }

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
        }
    }

    hcf();
}

#[panic_handler]
fn rust_panic(_info: &core::panic::PanicInfo) -> ! {
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
