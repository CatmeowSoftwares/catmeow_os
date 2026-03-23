use crate::{serial_println, terminal_println};
use core::arch::asm;
use lazy_static::lazy_static;
use x86_64::{
    instructions::{hlt, interrupts::int3},
    registers::control::Cr2,
    structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode},
};

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.divide_error.set_handler_fn(divide_error);
        idt.debug.set_handler_fn(debug);
        idt.non_maskable_interrupt
            .set_handler_fn(non_maskable_interrupt);
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt.overflow.set_handler_fn(overflow);
        idt.bound_range_exceeded
            .set_handler_fn(bound_range_exceeded);
        idt.invalid_opcode.set_handler_fn(invalid_opcode);
        idt.device_not_available
            .set_handler_fn(device_not_available);
        idt.double_fault.set_handler_fn(double_fault);
        idt.invalid_tss.set_handler_fn(invalid_tss);
        idt.segment_not_present.set_handler_fn(segment_not_present);
        idt.stack_segment_fault.set_handler_fn(stack_segment_fault);
        idt.general_protection_fault
            .set_handler_fn(general_protection_fault);
        idt.page_fault.set_handler_fn(page_fault);
        idt.x87_floating_point.set_handler_fn(x87_floating_point);
        idt.alignment_check.set_handler_fn(alignment_check);
        idt.machine_check.set_handler_fn(machine_check);
        idt.simd_floating_point.set_handler_fn(simd_floating_point);
        idt.virtualization.set_handler_fn(virtualization);
        idt.cp_protection_exception
            .set_handler_fn(cp_protection_exception);
        idt.hv_injection_exception
            .set_handler_fn(hv_injection_exception);
        idt.vmm_communication_exception
            .set_handler_fn(vmm_communication_exception);
        idt.security_exception.set_handler_fn(security_exception);
        idt[128].set_handler_fn(test);
        idt
    };
}

extern "x86-interrupt" fn debug(stack_frame: InterruptStackFrame) {
    serial_println!("debug: {:#?}", stack_frame);
}

extern "x86-interrupt" fn non_maskable_interrupt(stack_frame: InterruptStackFrame) {
    serial_println!("non_maskable_interrupt: {:#?}", stack_frame);
}
extern "x86-interrupt" fn overflow(stack_frame: InterruptStackFrame) {
    serial_println!("overflow: {:#?}", stack_frame);
}
extern "x86-interrupt" fn bound_range_exceeded(stack_frame: InterruptStackFrame) {
    serial_println!("bound_range_exceeded: {:#?}", stack_frame);
}
extern "x86-interrupt" fn invalid_opcode(stack_frame: InterruptStackFrame) {
    serial_println!("invalid_opcode: {:#?}", stack_frame);
    loop {}
}
extern "x86-interrupt" fn device_not_available(stack_frame: InterruptStackFrame) {
    serial_println!("device_not_available: {:#?}", stack_frame);
}
extern "x86-interrupt" fn x87_floating_point(stack_frame: InterruptStackFrame) {
    serial_println!("x87_floating_point: {:#?}", stack_frame);
}

extern "x86-interrupt" fn simd_floating_point(stack_frame: InterruptStackFrame) {
    serial_println!("simd_floating_point: {:#?}", stack_frame);
}
extern "x86-interrupt" fn virtualization(stack_frame: InterruptStackFrame) {
    serial_println!("virtualization: {:#?}", stack_frame);
}
extern "x86-interrupt" fn hv_injection_exception(stack_frame: InterruptStackFrame) {
    serial_println!("hv_injection_exception: {:#?}", stack_frame);
}

extern "x86-interrupt" fn test(stack_frame: InterruptStackFrame) {
    serial_println!("this was called: {:#?}", stack_frame);
}

extern "x86-interrupt" fn segment_not_present(stack_frame: InterruptStackFrame, err: u64) {
    serial_println!("segment_not_present: {:#?}, {:#?}", stack_frame, err);
}

extern "x86-interrupt" fn stack_segment_fault(stack_frame: InterruptStackFrame, err: u64) {
    serial_println!("stack_segment_fault: {:#?}, {:#?}", stack_frame, err);
}

extern "x86-interrupt" fn general_protection_fault(stack_frame: InterruptStackFrame, err: u64) {
    serial_println!("general_protection_fault: {:#?}, {:#?}", stack_frame, err);
    loop {
        hlt();
    }
}

extern "x86-interrupt" fn alignment_check(stack_frame: InterruptStackFrame, err: u64) {
    serial_println!("alignment_check: {:#?}, {:#?}", stack_frame, err);
}

extern "x86-interrupt" fn cp_protection_exception(stack_frame: InterruptStackFrame, err: u64) {
    serial_println!("cp_protection_exception: {:#?}, {:#?}", stack_frame, err);
}

extern "x86-interrupt" fn vmm_communication_exception(stack_frame: InterruptStackFrame, err: u64) {
    serial_println!(
        "vmm_communication_exception: {:#?}, {:#?}",
        stack_frame,
        err
    );
}

extern "x86-interrupt" fn security_exception(stack_frame: InterruptStackFrame, err: u64) {
    serial_println!("security_exception: {:#?}, {:#?}", stack_frame, err);
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
