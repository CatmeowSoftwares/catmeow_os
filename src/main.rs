#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(catmeow_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

use bootloader::{BootInfo, entry_point};
use catmeow_os::println;
use core::panic::PanicInfo;

//#[unsafe(no_mangle)]
//pub extern "C" fn _start(boot_info: &'static BootInfo)  -> ! {
entry_point!(kernel_main);
fn kernel_main(boot_info: &'static BootInfo) -> ! {

    catmeow_os::init();
    use x86_64::structures::paging::PageTable;
    use catmeow_os::memory::active_level_4_table;
    use x86_64::VirtAddr;

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let l4_table = unsafe { active_level_4_table(phys_mem_offset) };
    for (i, entry) in l4_table.iter().enumerate() {
        if !entry.is_unused() {
            println!("L4 Entry: {}: {:?}", i, entry);

            let phys = entry.frame().unwrap().start_address();
            let virt = phys.as_u64() + boot_info.physical_memory_offset;
            let ptr = VirtAddr::new(virt).as_mut_ptr();
            let l3_table: &PageTable = unsafe { &*ptr };
            for (i, entry) in l3_table.iter().enumerate() {
                if !entry.is_unused() {
                    println!("L3 Entry: {}: {:?}", i, entry);
                }
            }

        }
    }

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
    loop {asd
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
