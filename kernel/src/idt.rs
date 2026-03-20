use crate::{serial_println, terminal_println};
use core::arch::asm;
use lazy_static::lazy_static;
use x86_64::{
    instructions::interrupts::int3,
    registers::control::Cr2,
    structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode},
};

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.divide_error.set_handler_fn(divide_error);
        idt.debug.set_handler_fn(handler);
        idt.non_maskable_interrupt.set_handler_fn(handler);
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt.overflow.set_handler_fn(handler);
        idt.bound_range_exceeded.set_handler_fn(handler);
        idt.invalid_opcode.set_handler_fn(handler);
        idt.device_not_available.set_handler_fn(handler);
        idt.double_fault.set_handler_fn(double_fault);
        idt.invalid_tss.set_handler_fn(invalid_tss);
        idt.segment_not_present.set_handler_fn(invalid_tss);
        idt.stack_segment_fault.set_handler_fn(invalid_tss);
        idt.general_protection_fault.set_handler_fn(invalid_tss);
        idt.page_fault.set_handler_fn(page_fault);
        idt.x87_floating_point.set_handler_fn(handler);
        idt.alignment_check.set_handler_fn(invalid_tss);
        idt.machine_check.set_handler_fn(machine_check);
        idt.simd_floating_point.set_handler_fn(handler);
        idt.virtualization.set_handler_fn(handler);
        idt.cp_protection_exception.set_handler_fn(invalid_tss);
        idt.hv_injection_exception.set_handler_fn(handler);
        idt.vmm_communication_exception.set_handler_fn(invalid_tss);
        idt.security_exception.set_handler_fn(invalid_tss);
        idt
    };
}
extern "x86-interrupt" fn handler(stack_frame: InterruptStackFrame) {
    serial_println!("{:#?}", stack_frame);
}

extern "x86-interrupt" fn divide_error(stack_frame: InterruptStackFrame) {
    serial_println!("divide was used: {:#?}", stack_frame);
    let mut count = 0u128;

    loop {
        count += 1;
        terminal_println!("DIVIDE ERROR! count: {}", count);
    }
}

extern "x86-interrupt" fn double_fault(stack_frame: InterruptStackFrame, err: u64) -> ! {
    serial_println!("double fault: {:#?}, {}", stack_frame, err);
    loop {}
}

extern "x86-interrupt" fn invalid_tss(stack_frame: InterruptStackFrame, err: u64) {
    serial_println!("invalid tss: {:#?}, {:#?}", stack_frame, err);
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    serial_println!("{:#?}", stack_frame);
}

extern "x86-interrupt" fn page_fault(stack_frame: InterruptStackFrame, err: PageFaultErrorCode) {
    serial_println!("page fault: {:#?}, {:#?}", stack_frame, err);
    serial_println!("Accessed Address: {:?}", Cr2::read());
}
extern "x86-interrupt" fn machine_check(stack_frame: InterruptStackFrame) -> ! {
    serial_println!("machine check: {:#?}", stack_frame);
    loop {}
}
pub fn init_idt() {
    IDT.load();
    serial_println!("IDT initialized! :3");
    terminal_println!("IDT initialized! :3");
}
