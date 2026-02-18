#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(catmeow_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

use catmeow_os::println;
use core::panic::PanicInfo;

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    catmeow_os::init();

    /*
    fn stack_overflow() {
        stack_overflow();
    }
    stack_overflow();
     */
    //x86_64::instructions::interrupts::int3();

    #[cfg(test)]
    test_main();

    println!("it did not crash");
    catmeow_os::hlt_loop();
    /*
    loop {
        use catmeow_os::print;
        print!("-");
    }
     */
}

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    catmeow_os::hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    catmeow_os::test_panic_handler(info);
}
