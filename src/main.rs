#![no_std]
#![no_main]

#![feature(custom_test_frameworks)]
#![test_runner(catmeow_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use catmeow_os::{print, println, vga_buffer::test_this};


#[unsafe(no_mangle)]
pub extern  "C" fn _start() -> ! {
    

    catmeow_os::init();

    x86_64::instructions::interrupts::int3();
    
    #[cfg(test)]
    test_main();

    println!("it did not crash");
    loop {}
}







#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop{}
}


#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    catmeow_os::test_panic_handler(info);
}





