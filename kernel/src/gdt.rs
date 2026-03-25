use core::{
    arch::{asm, global_asm},
    cell::SyncUnsafeCell,
    f64::consts::PI,
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

struct Descriptor {
    base: u32,
    limit: u64,
    access_byte: u8,
    flags: u8,
}

#[repr(packed)]
struct GDTDesc {
    limit0_15: u16,
    base0_15: u16,
    base16_23: u8,
    access: u8,
    limit16_19: u8,
    other: u8,
    flags: u8,
    base24_31: u8,
}
static GDTR: SyncUnsafeCell<GlobalDescriptorTableRegister> =
    SyncUnsafeCell::new(GlobalDescriptorTableRegister::new());
static GDT: SyncUnsafeCell<[u64; 6]> = SyncUnsafeCell::new([0u64; 6]);

pub fn init_gdt() {
    /*
    create_descriptor(base, limit, flag);
    create_descriptor(base, limit, flag);
    create_descriptor(base, limit, flag);
    create_descriptor(base, limit, flag);
    create_descriptor(base, limit, flag);
    create_descriptor(base, limit, flag);
     */
    let gdt = unsafe { &mut *GDT.get() };
    gdt[0] = create_descriptor(0, 0, 0);
    gdt[1] = create_descriptor(0, 0x000FFFFF, (GDT_CODE_PL0));
    gdt[2] = create_descriptor(0, 0x000FFFFF, (GDT_DATA_PL0));
    gdt[3] = create_descriptor(0, 0x000FFFFF, (GDT_CODE_PL3));
    gdt[4] = create_descriptor(0, 0x000FFFFF, (GDT_DATA_PL3));

    let gdtr = GDTR.get();
    unsafe {
        (*gdtr).limit = (size_of_val(gdt) - 1) as u16;
        (*gdtr).base = gdt.as_ptr() as u64;
    }
    unsafe {
        lgdt(gdtr.as_mut_unchecked());
    }
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
/*
global_asm!(
    r#"
    .globl reload_cs
    reload_cs:
       #; Reload data segment registers
       MOV   AX, 0x10 #; 0x10 is a stand-in for your data segment
       MOV   DS, AX
       MOV   ES, AX
       MOV   FS, AX
       MOV   GS, AX
       MOV   SS, AX
       RET
    "#
);
global_asm!(
    r#"
    .globl reload_segments
    reload_segments:
       #; Reload CS register:
       PUSH 0x08                 #; Push code segment to stack, 0x08 is a stand-in for your code segment
       LEA RAX, [rel {rcs}] #; Load address of .reload_CS into RAX
       PUSH RAX                  #; Push this value to the stack
       RETFQ                     #; Perform a far return, RETFQ or LRETQ depending on syntax
    "#, rcs = sym reload_cs
);

unsafe extern "C" {
    fn reload_cs();
    fn reload_segments();
}
 */
fn reload_cs1() {
    unsafe {
        asm!(
            "
            mov ax 0x10
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
struct Segment {
    base: u64,
    limit: u64,
}

struct NullDescriptor {
    base: u64,
    limit: u64,
}
struct KernelCodeSegment {}
struct KernelDataSegment {}
struct UserDataSegment {}
struct UserCodeSegment {}
struct TaskStateSegment {}

fn encode_gdt_entry(target: *mut u8, source: Descriptor) {
    if source.limit > 0xFFFFF {
        panic!("GDT cannot encode limits larger than 0xFFFFF");
    }
    unsafe {
        *target.add(0) = (source.limit & 0xff) as u8;
        *target.add(1) = ((source.limit >> 8) & 0xff) as u8;
        *target.add(6) = ((source.limit >> 16) & 0xff) as u8;

        *target.add(2) = (source.base & 0xff) as u8;
        *target.add(3) = ((source.base >> 8) & 0xff) as u8;
        *target.add(4) = ((source.base >> 16) & 0xff) as u8;
        *target.add(7) = ((source.base >> 24) & 0xff) as u8;

        *target.add(5) = source.access_byte;

        *target.add(6) |= source.flags << 4;
    }
}

pub fn create_descriptor(base: u32, limit: u32, flag: u16) -> u64 {
    let mut descriptor = 0u64;

    descriptor = (limit & 0x000F0000) as u64;

    descriptor |= ((flag as u64) << 8) & 0x00F0FF00;
    descriptor |= (base >> 16) as u64 & 0x000000FF;
    descriptor |= base as u64 & 0xFF000000;
    descriptor <<= 32;

    descriptor |= (base << 16) as u64;
    descriptor |= (limit & 0x0000FFFF) as u64;

    serial_println!("0x{:016x}", descriptor);
    descriptor
}
