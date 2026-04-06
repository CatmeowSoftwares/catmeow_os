use core::arch::asm;
use core::cell::SyncUnsafeCell;

use crate::serial_println;
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
static IDTR: SyncUnsafeCell<IDTR> = SyncUnsafeCell::new(IDTR::new());

const IDT_MAX_DESCRIPTORS: usize = 256;
static IDT: SyncUnsafeCell<[IDTEntry; IDT_MAX_DESCRIPTORS]> =
    SyncUnsafeCell::new([IDTEntry::new(); IDT_MAX_DESCRIPTORS]);

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

fn idt_set_descriptor(vector: u8, isr: *mut u8, flags: u8) {
    let idt = unsafe { &mut *IDT.get() };
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
    let idtr = unsafe { &mut *IDTR.get() };
    let idt = unsafe { &mut *IDT.get() };
    idtr.base = idt.as_ptr() as u64;
    idtr.limit = (IDT_MAX_DESCRIPTORS * size_of::<IDTEntry>() - 1) as u16;
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

    lidt(idtr);
    enable_interrupts();
    serial_println!("IDT INITIALIZED");
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

fn divide_error_handler() {
    panic!("divide error!");
}
fn debug_exception_handler() {
    panic!("debug exception!");
}

fn nmi_interrupt_handler() {
    panic!("nmi interrupt!");
}
fn breakpoint_handler() {
    panic!("breakpoint!");
}
fn overflow_handler() {
    panic!("overflow!");
}
fn bound_range_exceeded_handler() {
    panic!("bound range exceeded!");
}
fn invalid_opcode_handler() {
    panic!("invalid opcode!");
}
fn device_not_available_handler() {
    panic!("device not available!");
}
fn double_fault_handler() {
    panic!("double fault!");
}
fn coprocessor_segment_overrun_handler() {
    panic!("coprocessor segment overrun!");
}
fn invalid_tss_handler() {
    panic!("invalid tss!");
}
fn segment_not_present_handler() {
    panic!("segment not present!");
}

fn stack_segment_fault_handler() {
    panic!("stack segment fault!");
}

fn general_protection_handler() {
    panic!("general protection!");
}

fn page_fault_handler() {
    panic!("page fault!");
}

fn x87_fpu_floating_point_error_handler() {
    panic!("x87_fpu_floating_point_error!");
}
fn alignment_check_handler() {
    panic!("alignment check!");
}
fn machine_check_handler() {
    panic!("machine check!");
}
fn simd_floating_point_exception_handler() {
    panic!("simd floating point exception!");
}
fn virtualization_exception_handler() {
    panic!("virtualization exeption!");
}
fn control_protection_exception_handler() {
    panic!("control_protection_exception!");
}
fn hypervisor_injection_exception_handler() {
    panic!("hypervisor injection exception!");
}
fn vmm_communication_exception_handler() {
    panic!("vmm communication exception!");
}
fn security_exception_handler() {
    panic!("security exception!");
}
