use core::arch::asm;

use spin::Mutex;

use crate::{serial_println, terminal_println};
#[repr(C, packed)]
struct IDTR {
    limit: u16,
    base: u64,
}
impl IDTR {
    const fn new() -> Self {
        Self { limit: 0, base: 0 }
    }
}
static IDTR: Mutex<IDTR> = Mutex::new(IDTR::new());

const IDT_MAX_DESCRIPTORS: usize = 256;
static IDT: Mutex<[IDTEntry; IDT_MAX_DESCRIPTORS]> =
    Mutex::new([IDTEntry::new(); IDT_MAX_DESCRIPTORS]);

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
struct IDTEntry {
    isr_low: u16,
    kernel_cs: u16,
    ist: u8,
    attributes: u8,
    isr_mid: u16,
    isr_high: u32,
    reserved: u32,
}

impl IDTEntry {
    const fn new() -> Self {
        Self {
            isr_low: 0,
            kernel_cs: 0,
            ist: 0,
            attributes: 0,
            isr_mid: 0,
            isr_high: 0,
            reserved: 0,
        }
    }
}

const GDT_OFFSET_KERNEL_CODE: u16 = 0x08;

#[derive(Debug)]
#[repr(C)]
pub(crate) struct InterruptStackFrame {
    ip: usize,
    cs: usize,
    flags: usize,
    sp: usize,
    ss: usize,
}

fn idt_set_descriptor(vector: u8, isr: *mut u8, flags: u8) {
    let mut idt = IDT.lock();
    let descriptor = &mut idt[vector as usize];
    descriptor.isr_low = isr as u16 & 0xffff;
    descriptor.kernel_cs = GDT_OFFSET_KERNEL_CODE;
    descriptor.ist = 0;
    descriptor.attributes = flags;
    descriptor.isr_mid = ((isr as u64 >> 16) & 0xffff) as u16;
    descriptor.isr_high = ((isr as u64 >> 32) & 0xffffffff) as u32;
    descriptor.reserved = 0;
}

pub fn init_idt() {
    {
        let mut idtr = IDTR.lock();
        let idt = IDT.lock();
        idtr.base = idt.as_ptr() as u64;
        idtr.limit = (IDT_MAX_DESCRIPTORS * size_of::<IDTEntry>() - 1) as u16;
    }
    for i in 0..IDT_MAX_DESCRIPTORS {
        idt_set_descriptor(i as u8, exception_handler as *mut u8, 0x8e);
    }
    idt_set_descriptor(0x20, crate::pit::timer_interrupt_handler as *mut u8, 0x8e);
    idt_set_descriptor(0, divide_error_handler as *mut u8, 0x8e);
    idt_set_descriptor(1, debug_exception_handler as *mut u8, 0x8e);
    idt_set_descriptor(2, nmi_interrupt_handler as *mut u8, 0x8e);
    idt_set_descriptor(3, breakpoint_handler as *mut u8, 0x8e);
    idt_set_descriptor(4, overflow_handler as *mut u8, 0x8e);
    idt_set_descriptor(5, bound_range_exceeded_handler as *mut u8, 0x8e);
    idt_set_descriptor(6, invalid_opcode_handler as *mut u8, 0x8e);
    idt_set_descriptor(7, device_not_available_handler as *mut u8, 0x8e);
    idt_set_descriptor(8, double_fault_handler as *mut u8, 0x8e);
    idt_set_descriptor(9, coprocessor_segment_overrun_handler as *mut u8, 0x8e);
    idt_set_descriptor(10, invalid_tss_handler as *mut u8, 0x8e);
    idt_set_descriptor(11, segment_not_present_handler as *mut u8, 0x8e);
    idt_set_descriptor(12, stack_segment_fault_handler as *mut u8, 0x8e);
    idt_set_descriptor(13, general_protection_handler as *mut u8, 0x8e);
    idt_set_descriptor(14, page_fault_handler as *mut u8, 0x8e);
    idt_set_descriptor(16, x87_fpu_floating_point_error_handler as *mut u8, 0x8e);
    idt_set_descriptor(17, alignment_check_handler as *mut u8, 0x8e);
    idt_set_descriptor(18, machine_check_handler as *mut u8, 0x8e);
    idt_set_descriptor(19, simd_floating_point_exception_handler as *mut u8, 0x8e);
    idt_set_descriptor(20, virtualization_exception_handler as *mut u8, 0x8e);
    idt_set_descriptor(21, control_protection_exception_handler as *mut u8, 0x8e);
    idt_set_descriptor(28, hypervisor_injection_exception_handler as *mut u8, 0x8e);
    idt_set_descriptor(29, vmm_communication_exception_handler as *mut u8, 0x8e);
    idt_set_descriptor(30, security_exception_handler as *mut u8, 0x8e);

    let idtr = IDTR.lock();
    lidt(&*idtr);
    enable_interrupts();
}

pub fn disable_interrupts() {
    unsafe {
        asm!("cli", options(nostack));
    }
}
pub fn enable_interrupts() {
    unsafe {
        asm!("sti", options(nostack));
    }
}
fn lidt(idtr: &IDTR) {
    unsafe {
        asm!("lidt [{}]", in(reg) idtr, options(readonly, nostack, preserves_flags));
    }
}

unsafe extern "x86-interrupt" fn exception_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    panic!("exception!: {:#x?}, err code: {}", stack_frame, error_code);
}

unsafe extern "x86-interrupt" fn divide_error_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    panic!(
        "divide error: {:#x?}, err code: {}",
        stack_frame, error_code
    );
}
unsafe extern "x86-interrupt" fn debug_exception_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    panic!(
        "debug exception: {:#x?}, err code: {}",
        stack_frame, error_code
    );
}

unsafe extern "x86-interrupt" fn nmi_interrupt_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    panic!(
        "nmi interrupt: {:#x?}, err code: {}",
        stack_frame, error_code
    );
}
unsafe extern "x86-interrupt" fn breakpoint_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    panic!("breakpoint: {:#x?}, err code: {}", stack_frame, error_code);
}
unsafe extern "x86-interrupt" fn overflow_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    panic!("overflow: {:#x?}, err code: {}", stack_frame, error_code);
}
unsafe extern "x86-interrupt" fn bound_range_exceeded_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    panic!(
        "bound range exceeded: {:#x?}, err code: {}",
        stack_frame, error_code
    );
}
unsafe extern "x86-interrupt" fn invalid_opcode_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    panic!(
        "invalid opcode: {:#x?}, err code: {}",
        stack_frame, error_code
    );
}
unsafe extern "x86-interrupt" fn device_not_available_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    panic!(
        "device not available: {:#x?}, err code: {}",
        stack_frame, error_code
    );
}
unsafe extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    panic!(
        "double fault: {:#x?}, err code: {}",
        stack_frame, error_code
    );
}
unsafe extern "x86-interrupt" fn coprocessor_segment_overrun_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    panic!(
        "coprocessor segment overrun: {:#x?}, err code: {}",
        stack_frame, error_code
    );
}
unsafe extern "x86-interrupt" fn invalid_tss_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    panic!("invalid tss: {:#x?}, err code: {}", stack_frame, error_code);
}
unsafe extern "x86-interrupt" fn segment_not_present_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    panic!(
        "segment not present: {:#x?}, err code: {}",
        stack_frame, error_code
    );
}

unsafe extern "x86-interrupt" fn stack_segment_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    panic!(
        "stack segment fault: {:#x?}, err code: {}",
        stack_frame, error_code
    );
}

unsafe extern "x86-interrupt" fn general_protection_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    panic!(
        "general protection: {:#x?}, err code: {}",
        stack_frame, error_code
    );
}

unsafe extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    panic!("page fault: {:#x?}, err code: {}", stack_frame, error_code);
}

unsafe extern "x86-interrupt" fn x87_fpu_floating_point_error_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    panic!(
        "x87_fpu_floating_point_error: {:#x?}, err code: {}",
        stack_frame, error_code
    );
}
unsafe extern "x86-interrupt" fn alignment_check_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    panic!(
        "alignment check: {:#x?}, err code: {}",
        stack_frame, error_code
    );
}
unsafe extern "x86-interrupt" fn machine_check_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    panic!(
        "machine check: {:#x?}, err code: {}",
        stack_frame, error_code
    );
}
unsafe extern "x86-interrupt" fn simd_floating_point_exception_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    panic!(
        "simd floating point exception: {:#x?}, err code: {}",
        stack_frame, error_code
    );
}
unsafe extern "x86-interrupt" fn virtualization_exception_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    panic!(
        "virtualization exeption: {:#x?}, err code: {}",
        stack_frame, error_code
    );
}
unsafe extern "x86-interrupt" fn control_protection_exception_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    panic!(
        "control_protection_exception: {:#x?}, err code: {}",
        stack_frame, error_code
    );
}
unsafe extern "x86-interrupt" fn hypervisor_injection_exception_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    panic!(
        "hypervisor injection exception: {:#x?}, err code: {}",
        stack_frame, error_code
    );
}
unsafe extern "x86-interrupt" fn vmm_communication_exception_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    panic!(
        "vmm communication exception: {:#x?}, err code: {}",
        stack_frame, error_code
    );
}
unsafe extern "x86-interrupt" fn security_exception_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    panic!(
        "security exception: {:#x?}, err code: {}",
        stack_frame, error_code
    );
}
