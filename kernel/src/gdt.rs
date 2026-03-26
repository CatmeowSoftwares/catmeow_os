use core::{
    arch::{asm, global_asm},
    cell::SyncUnsafeCell,
};

use crate::serial_println;
const GDTBASE: u64 = 0x00000800;

const fn seg_desktype(x: u16) -> u16 {
    x << 0x04
}

const fn seg_present(x: u16) -> u16 {
    x << 0x07
}
const fn seg_savl(x: u16) -> u16 {
    x << 0x0c
}
const fn seg_long(x: u16) -> u16 {
    x << 0x0d
}

const fn seg_size(x: u16) -> u16 {
    x << 0x0e
}
const fn seg_gran(x: u16) -> u16 {
    x << 0x0f
}
const fn seg_priv(x: u16) -> u16 {
    (x & 0x03) << 0x05
}

const SEG_DATA_RD: u16 = 0x00; // Read-Only
const SEG_DATA_RDA: u16 = 0x01; // Read-Only, accessed
const SEG_DATA_RDWR: u16 = 0x02; // Read/Write
const SEG_DATA_RDWRA: u16 = 0x03; // Read/Write, accessed
const SEG_DATA_RDEXPD: u16 = 0x04; // Read-Only, expand-down
const SEG_DATA_RDEXPDA: u16 = 0x05; // Read-Only, expand-down, accessed
const SEG_DATA_RDWREXPD: u16 = 0x06; // Read/Write, expand-down
const SEG_DATA_RDWREXPDA: u16 = 0x07; // Read/Write, expand-down, accessed
const SEG_CODE_EX: u16 = 0x08; // Execute-Only
const SEG_CODE_EXA: u16 = 0x09; // Execute-Only, accessed
const SEG_CODE_EXRD: u16 = 0x0A; // Execute/Read
const SEG_CODE_EXRDA: u16 = 0x0B; // Execute/Read, accessed
const SEG_CODE_EXC: u16 = 0x0C; // Execute-Only, conforming
const SEG_CODE_EXCA: u16 = 0x0D; // Execute-Only, conforming, accessed
const SEG_CODE_EXRDC: u16 = 0x0E; // Execute/Read, conforming
const SEG_CODE_EXRDCA: u16 = 0x0F; // Execute/Read, conforming, accessed

const GDT_CODE_PL0: u16 = seg_desktype(1)
    | seg_present(1)
    | seg_savl(0)
    | seg_long(1)
    | seg_size(0)
    | seg_gran(1)
    | seg_priv(0)
    | SEG_CODE_EXRD;

const GDT_DATA_PL0: u16 = seg_desktype(1)
    | seg_present(1)
    | seg_savl(0)
    | seg_long(1)
    | seg_size(0)
    | seg_gran(1)
    | seg_priv(0)
    | SEG_DATA_RDWR;

const GDT_CODE_PL3: u16 = seg_desktype(1)
    | seg_present(1)
    | seg_savl(0)
    | seg_long(1)
    | seg_size(0)
    | seg_gran(1)
    | seg_priv(3)
    | SEG_CODE_EXRD;

const GDT_DATA_PL3: u16 = seg_desktype(1)
    | seg_present(1)
    | seg_savl(0)
    | seg_long(0)
    | seg_size(1)
    | seg_gran(1)
    | seg_priv(3)
    | SEG_DATA_RDWR;
pub type GDT = GlobalDescriptorTableRegister;
#[repr(C, packed)]
pub struct GlobalDescriptorTableRegister {
    limit: u16,
    base: u64,
}

impl GlobalDescriptorTableRegister {
    const fn new() -> Self {
        Self { limit: 0, base: 0 }
    }
}

static GDTR: SyncUnsafeCell<GlobalDescriptorTableRegister> =
    SyncUnsafeCell::new(GlobalDescriptorTableRegister::new());
static GDT: SyncUnsafeCell<[u64; 6]> = SyncUnsafeCell::new([0u64; 6]);

pub fn init_gdt() {
    let gdt = unsafe { &mut *GDT.get() };
    gdt[0] = create_descriptor(0, 0, 0);
    gdt[1] = create_descriptor(0, 0x000FFFFF, GDT_CODE_PL0);
    gdt[2] = create_descriptor(0, 0x000FFFFF, GDT_DATA_PL0);
    gdt[3] = create_descriptor(0, 0x000FFFFF, GDT_CODE_PL3);
    gdt[4] = create_descriptor(0, 0x000FFFFF, GDT_DATA_PL3);

    let gdtr = unsafe { &mut *GDTR.get() };
    gdtr.limit = (size_of_val(gdt) - 1) as u16;
    gdtr.base = gdt.as_ptr() as u64;
    lgdt(gdtr);
    reload_segments();
    serial_println!("GDT INITIALIZED")
}

fn reload_segments() {
    unsafe {
        asm!(
            "
            push 0x08
            lea rax, [rip + 2f]
            push rax
            retfq
        2:
            mov ax, 0x10
            mov ds, ax
            mov es, ax
            mov fs, ax
            mov gs, ax
            mov ss, ax
            "
        );
    }
}

fn lgdt(gdt: *const GlobalDescriptorTableRegister) {
    unsafe { asm!("lgdt [{}]", in(reg) gdt, options(readonly, nostack, preserves_flags)) }
}

pub fn create_descriptor(base: u32, limit: u32, flag: u16) -> u64 {
    let mut descriptor;

    descriptor = (limit & 0x000F0000) as u64;

    descriptor |= ((flag as u64) << 8) & 0x00F0FF00;
    descriptor |= (base >> 16) as u64 & 0x000000FF;
    descriptor |= base as u64 & 0xFF000000;
    descriptor <<= 32;

    descriptor |= (base << 16) as u64;
    descriptor |= (limit & 0x0000FFFF) as u64;

    //serial_println!("0x{:016x}", descriptor);
    descriptor
}
