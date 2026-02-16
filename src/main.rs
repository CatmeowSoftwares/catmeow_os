#![no_std]
#![no_main]
use core::panic::PanicInfo;

use crate::vga_buffer::{Writer, print_something};


#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop{}
}
mod vga_buffer;

static HELLO: &[u8] = b"Hello World!";

#[unsafe(no_mangle)]
pub extern  "C" fn _start() -> ! {
    println!("{}", 1.0f64/3.0f64);
    println!("print this");
    panic!("meow :3");
    loop {}
}




