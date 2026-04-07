use crate::terminal_println;
use core::{cell::SyncUnsafeCell, ffi::c_char, fmt::Write, ptr::null_mut};
use flanterm::sys::flanterm_context;
use limine::framebuffer::Framebuffer;
struct Terminal {
    ctx: *mut flanterm_context,
}
unsafe impl Sync for Terminal {}
unsafe impl Send for Terminal {}
impl Write for Terminal {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let ctx = self.ctx;
        unsafe {
            flanterm::sys::flanterm_write(ctx, s.as_ptr().cast::<c_char>(), s.len());
        }
        Ok(())
    }
}
static TERMINAL: SyncUnsafeCell<Terminal> = SyncUnsafeCell::new(Terminal { ctx: null_mut() });
pub fn init_terminal(framebuffer: Framebuffer) {
    {
        let terminal = unsafe { &mut *TERMINAL.get() };
        unsafe {
            terminal.ctx = flanterm::sys::flanterm_fb_init(
                None,
                None,
                framebuffer.addr() as *mut u32,
                framebuffer.width() as usize,
                framebuffer.height() as usize,
                framebuffer.pitch() as usize,
                framebuffer.red_mask_size(),
                framebuffer.red_mask_shift(),
                framebuffer.green_mask_size(),
                framebuffer.green_mask_shift(),
                framebuffer.blue_mask_size(),
                framebuffer.blue_mask_shift(),
                null_mut(),
                null_mut(),
                null_mut(),
                null_mut(),
                null_mut(),
                null_mut(),
                null_mut(),
                null_mut(),
                0,
                0,
                1,
                0,
                0,
                0,
            );
        }
    }
    terminal_println!("Terminal initialized! :3");
}

#[doc(hidden)]
pub fn _print(args: ::core::fmt::Arguments) {
    let terminal = unsafe { &mut *TERMINAL.get() };
    terminal
        .write_fmt(args)
        .expect("failed to write in terminal");
}

#[macro_export]
macro_rules! terminal_print {
    () => {
        $crate::print!("\n")
    };
    ($($arg:tt)*) => {{
        $crate::terminal::_print(format_args!($($arg)*));
    }};
}

#[macro_export]
macro_rules! terminal_println {
    () => ($crate::terminal_print!("\n"));
    ($fmt:expr) => ($crate::terminal_print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => ($crate::terminal_print!(
        concat!($fmt, "\n"), $($arg)*));
}
