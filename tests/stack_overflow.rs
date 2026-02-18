#![no_main]
#![no_std]
#![feature(abi_x86_interrupt)]

use core::panic::PanicInfo;

use catmeow_os::{exit_qemu, serial_print, serial_println};

#[unsafe(no_mangle)]
pub fn _start() -> ! {
    serial_print!("stack_overflow::stack_overflow...\t");

    catmeow_os::gdt::init();
    init_test_idt();

    stack_overflow();
    panic!("Execution continued after stack overflow");
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    catmeow_os::test_panic_handler(info);
}

#[allow(unconditional_recursion)]
fn stack_overflow() {
    stack_overflow();
    volatile::Volatile::new(0).read();
}

use lazy_static::lazy_static;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

lazy_static! {
    static ref TEST_IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        unsafe {
            idt.double_fault
                .set_handler_fn(test_double_fault_handler)
                .set_stack_index(catmeow_os::gdt::DOUBLE_FAULT_IST_INDEX);
        }
        idt
    };
}
fn init_test_idt() {
    TEST_IDT.load();
}

use catmeow_os::QemuExitCode;

extern "x86-interrupt" fn test_double_fault_handler(
    _stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    serial_println!("[ok]");
    exit_qemu(QemuExitCode::Success);
    loop {}
}
