use core::ptr::null_mut;

use limine::framebuffer::Framebuffer;
use spin::Mutex;
static SCREEN: Mutex<Option<Screen>> = Mutex::new(None);

struct Screen {
    addr: *mut u8,
    width: u64,
    height: u64,
    pitch: u64,
}
unsafe impl Send for Screen {}
unsafe impl Sync for Screen {}
pub fn init_screen(framebuffer: &Framebuffer) {
    let mut screen = SCREEN.lock();
    *screen = Some(Screen {
        addr: framebuffer.addr(),
        width: framebuffer.width(),
        height: framebuffer.height(),
        pitch: framebuffer.pitch(),
    });
}

pub fn put_pixel(x: u32, y: u32, color: u32) {
    let screen = SCREEN.lock();
    if let Some(screen) = &*screen {
        let pixel_offset = y as u64 * screen.pitch + x as u64 * 4;

        unsafe {
            screen
                .addr
                .add(pixel_offset as usize)
                .cast::<u32>()
                .write(color)
        };
    }
}
pub fn put_rect(x: u32, y: u32, width: u32, height: u32, color: u32) {
    for iter_x in 0..width {
        for iter_y in 0..height {
            put_pixel(x + iter_x, y + iter_y, color);
        }
    }
}
pub fn put_line(x1: u32, y1: u32, x2: u32, y2: u32, color: u32) {
    let dx = x2 - x1;
    let dy = y2 - y1;
    let m = dy / dx;
    for x in x1..x2 {
        let y = m * (x - x1) + y1;
        put_pixel(x, y, color);
    }
}

struct Color(u8, u8, u8);

impl Color {
    pub fn new(red: u8, green: u8, blue: u8) -> Self {
        Self(red, green, blue)
    }
}
