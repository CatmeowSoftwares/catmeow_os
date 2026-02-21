#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(catmeow_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

use bootloader::{BootInfo, entry_point};
use catmeow_os::{memory::{BootInfoFrameAllocator, EmptyAllocator, init}, println};
use x86_64::structures::paging::{Page, Size4KiB, Translate, page};
use core::panic::PanicInfo;

//#[unsafe(no_mangle)]
//pub extern "C" fn _start(boot_info: &'static BootInfo)  -> ! {
entry_point!(kernel_main);
fn kernel_main(boot_info: &'static BootInfo) -> ! {

    catmeow_os::init();
    use x86_64::structures::paging::PageTable;
    use x86_64::VirtAddr;

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);

    let mut mapper = unsafe { init(phys_mem_offset) };
    let addresses = [
        0xb8000,
        0x201008,
        0x0100_0020_1a10,
        boot_info.physical_memory_offset,
    ];
    
    for &address in &addresses {
        let virt = VirtAddr::new(address);
        let phys = mapper.translate_addr(virt);//unsafe { translate_addr(virt, phys_mem_offset) };
        println!("{:?} -> {:?}", virt, phys);
    }


    let page: Page<Size4KiB> = Page::containing_address(VirtAddr::new(0));
    let mut empty_allocator = EmptyAllocator;

    let page_ptr: *mut u64 = page.start_address().as_mut_ptr();
    unsafe { page_ptr.offset(400).write_volatile(0x_f021_f077_f065_f04e);}



    let mut frame_allocator= unsafe {
        BootInfoFrameAllocator::init(&boot_info.memory_map)
    };

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
