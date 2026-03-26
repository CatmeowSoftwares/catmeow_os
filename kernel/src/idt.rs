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

fn exception_handler() -> ! {
    serial_println!("exception!");
    loop {
        unsafe {
            asm!("hlt");
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

static VECTORS: SyncUnsafeCell<[bool; IDT_MAX_DESCRIPTORS]> =
    SyncUnsafeCell::new([false; IDT_MAX_DESCRIPTORS]);

static ISR_STUB_TABLE: SyncUnsafeCell<[u64; IDT_MAX_DESCRIPTORS]> =
    SyncUnsafeCell::new([0u64; IDT_MAX_DESCRIPTORS]);

pub fn init_idt() {
    let idtr = unsafe { &mut *IDTR.get() };
    let idt = unsafe { &mut *IDT.get() };
    unsafe {
        idtr.base = idt.as_ptr() as u64;
        idtr.limit = (IDT_MAX_DESCRIPTORS * size_of::<IDTEntry>() - 1) as u16;
        let isr_stub_table = &mut *ISR_STUB_TABLE.get();
        let vectors = &mut *VECTORS.get();
        for vector in 0..32 {
            //idt_set_descriptor(vector, isr_stub_table[vector as usize] as *mut u8, 0x8e);
            idt_set_descriptor(vector, exception_handler as *mut u8, 0x8e);
            vectors[vector as usize] = true;
        }
        lidt(idtr);
        enable_interrupts();
        serial_println!("IDT INITIALIZED");
    }
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
