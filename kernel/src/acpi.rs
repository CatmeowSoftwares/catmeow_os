use crate::{
    idt::{disable_interrupts, enable_interrupts},
    memory::{
        PhysicalPointer,
        pmm::{get_hhdm_offset, page_align},
        vmm::{map_page, unmap_page},
    },
    pit::{sleep, wait},
    serial, serial_println, terminal_println,
};
use bitfields::bitfield;
use core::{
    str::from_utf8,
    sync::atomic::{AtomicU32, AtomicU64},
};
#[repr(C, packed)]
#[derive(Clone, Copy)]
struct Rsdp {
    signature: [u8; 8],
    checksum: u8,
    oem_id: [u8; 6],
    revision: u8,
    rsdt_address: u32,
}

#[derive(Clone, Copy)]
#[repr(C)]
struct AcpiSdtHeader {
    signature: [u8; 4],
    length: u32,
    revision: u8,
    checksum: u8,
    oem_id: [u8; 6],
    oem_table_id: [u8; 8],
    oem_revision: u32,
    creator_id: u32,
    creator_revision: u32,
}
#[derive(Clone, Copy)]
#[repr(C, packed)]
struct Rsdt {
    h: AcpiSdtHeader,
    pointer_to_other_sdt: *const u32,
}
#[repr(C, packed)]
#[derive(Clone, Copy)]
struct LocalApicOverride {
    entry_type: u8,
    record_length: u8,
    reserved: u16,
    local_apic_addr: u64,
}
#[repr(C, packed)]
#[derive(Clone, Copy)]
struct ProcessorLocalApic {
    entry_type: u8,
    record_length: u8,
    acpi_processor_id: u8,
    acpi_id: u8,
    flags: u32,
}
#[repr(C, packed)]
#[derive(Clone, Copy)]
struct IoApic {
    entry_type: u8,
    record_length: u8,
    io_apic_id: u8,
    reserved: u8,
    io_apic_addr: u32,
    global_system_interrupt_base: u32,
}
#[repr(C, packed)]
#[derive(Clone, Copy)]
struct IoApicInterruptSourceOverride {
    entry_type: u8,
    record_length: u8,
    bus_source: u8,
    irq_source: u8,
    global_system_interrupt: u32,
    flags: u16,
}
#[repr(C, packed)]
#[derive(Clone, Copy)]
struct MadtEntry {
    entry_type: u8,
    record_length: u8,
}
#[repr(C, packed)]
#[derive(Clone, Copy)]
struct Madt {
    head: AcpiSdtHeader,
    local_acpi_addr: u32,
    flags: u32,
}
#[derive(Clone, Copy)]
#[bitfield(u64)]
struct IoApicRegister {
    #[bits(7)]
    interrupt_vector: u8,
    #[bits(3)]
    delivery_mode: u8,
    destination_mode: bool,
    interrupt_sent: bool,
    polarity: bool,
    idk: bool,
    trigger_mode: bool,
    interrupt_mask: bool,
    #[bits(40)]
    reserved: u64,
    destination_field: u8,
}

#[bitfield(u64)]
struct IoRedTbl {
    vector: u8,
    #[bits(3)]
    delivery_mode: u8,
    destination_mode: bool,
    delivery_status: bool,
    pin_polarity: bool,
    remote_irr: bool,
    trigger_mode: bool,
    mask: bool,
    #[bits(39)]
    reserved: u64,
    destination: u8,
}

const LAPIC_ID_REGISTER: u32 = 0x020;
const LAPIC_VERSION_REGISTER: u32 = 0x030;
const TASK_PRIORITY_REGISTER: u32 = 0x080;
const ARBITRATION_PRIORITY_REGISTER: u32 = 0x090;
const PROCESSOR_PRIORITY_REGISTER: u32 = 0x0a0;
const EOI: u32 = 0x0b0;
const REMOTE_READ_REGISTER: u32 = 0x0c0;
const LOGICAL_DESTINATION_REGISTER: u32 = 0x0d0;
const DESTINATION_FORMAT_REGISTER: u32 = 0x0e0;
const SPURIOUS_INTERRUPT_VECTOR_REGISTER: u32 = 0x0f0;
const IN_SERVICE_REGISTER: u32 = 0x100;
const TRIGGER_MODE_REGISTER: u32 = 0x180;
const INTERRUPT_REQUEST_REGISTER: u32 = 0x200;
const ERROR_STATUS_REGISTER: u32 = 0x280;
const LVT_CORRECTED_MACHINE_CHECK_INTERRUPT_REGISTER: u32 = 0x2f0;
const INTERRUPT_COMMAND_REGISTER: u32 = 0x300;
const LVT_TIMER_REGISTER: u32 = 0x320;
const LVT_THERMAL_SENSOR_REGISTER: u32 = 0x330;
const LVT_PERFORMANCE_MONITORING_COUNTERS_REGISTER: u32 = 0x340;
const LVT_LINT0_REGISTER: u32 = 0x350;
const LVT_LINT1_REGISTER: u32 = 0x360;
const LVT_ERROR_REGISTER: u32 = 0x370;
const INITIAL_COUNT_REGISTER: u32 = 0x380;
const CURRENT_COUNT_REGISTER: u32 = 0x390;
const DIVIDE_CONFIGURATION_REGISTER: u32 = 0x3e0;
fn write_ioapic_register(apic_base: *mut u32, offset: u8, val: u32) {
    unsafe {
        *apic_base = offset as u32;
        *apic_base.add(0x10) = val;
    }
}
fn read_ioapic_register(apic_base: *mut u32, offset: u8) -> u32 {
    unsafe {
        *apic_base = offset as u32;
        *apic_base.add(0x10)
    }
}
fn write32(wher: u32, value: u32) {
    let mut ptr = PhysicalPointer::<u32>::new(wher as usize);
    *ptr = value;
}
fn read32(wher: u32) -> u32 {
    let ptr = PhysicalPointer::<u32>::new(wher as usize);
    *ptr
}
fn apic_start_timer() {
    let pit_prepare_sleep = |x| x;
    let pit_perform_sleep = || {};
    write32(DIVIDE_CONFIGURATION_REGISTER, 0x3);
    pit_prepare_sleep(10000);
    write32(INITIAL_COUNT_REGISTER, 0xFFFFFFFF);
    pit_perform_sleep();
    write32(INITIAL_COUNT_REGISTER, 0);
    let ticks_in_10ms: u32 = 0xFFFFFFFF - read32(CURRENT_COUNT_REGISTER);
    write32(LVT_TIMER_REGISTER, read32(LVT_TIMER_REGISTER) & (1 << 17));
    write32(DIVIDE_CONFIGURATION_REGISTER, 0x3);
    write32(INITIAL_COUNT_REGISTER, ticks_in_10ms);
}

fn get_vendor_id() {
    let (_, ebx, ecx, edx) = cpuid(0);
    terminal_println!(
        "{}{}{}",
        from_utf8(&ebx.to_le_bytes()).unwrap(),
        from_utf8(&edx.to_le_bytes()).unwrap(),
        from_utf8(&ecx.to_le_bytes()).unwrap(),
    );
}

pub fn init_acpi(rsdp_address: usize) {
    get_vendor_id();
    let local_apic_addr: u32;
    unsafe {
        core::arch::asm!(
            "
            mov rcx, 0x1B
            rdmsr",
            out(    "eax") local_apic_addr
        );
    }
    terminal_println!("local_apic_addr after rdmsr: 0x{:x}", local_apic_addr);

    let sivr = local_apic_addr + 0xf0;
    unsafe {
        serial::outb(0x20 + 1, 0xff);
        serial::outb(0xA0 + 1, 0xff);
        serial::outb(0x20, 0x20);
    }

    terminal_println!("thing start");
    let phys_ptr: PhysicalPointer<Rsdp> = PhysicalPointer::new(rsdp_address);

    let a = unsafe { phys_ptr.read() };
    terminal_println!("thing signature: {}", from_utf8(&a.signature[..]).unwrap());
    terminal_println!("thing checksum: {}", a.checksum);
    let rsdt_addr = a.rsdt_address;

    let rsdt: PhysicalPointer<AcpiSdtHeader> = PhysicalPointer::new(rsdt_addr as usize);
    terminal_println!("rsdt ptr b: 0x{:x}", rsdt.address());
    let rsdt_read = unsafe { rsdt.read() };
    terminal_println!("rsdt ptr a: 0x{:x}", rsdt.address());

    terminal_println!("length:{}", rsdt_read.length);
    terminal_println!("revision:{}", rsdt_read.revision);
    terminal_println!(
        "rsdt signature: {}",
        from_utf8(&rsdt_read.signature[..]).unwrap()
    );
    terminal_println!("ptr: 0x{:x}", rsdt.address());
    terminal_println!("start finding signature");
    terminal_println!("rsdt addr: 0x{:x}", rsdt.address());
    let a = find_signature(&rsdt, b"APIC");
    if let Some(madt_ptr) = a {
        let madt_addr = madt_ptr.address();
        let madt: PhysicalPointer<Madt> = PhysicalPointer::new(madt_addr);
        madt.local_acpi_addr;
        let mut current_entry: PhysicalPointer<MadtEntry>;
        serial_println!(
            "madt signature: {}",
            from_utf8(&madt.head.signature[..]).unwrap()
        );

        let mut current = madt_addr + size_of::<Madt>();
        let end = madt_addr + madt.head.length as usize;
        serial_println!("start: {}", current);
        serial_println!("end: {}", end);
        let entry_type_test = PhysicalPointer::<u8>::new(madt_addr + size_of::<Madt>());
        serial_println!("entry type: {}", *entry_type_test);
        let record_len_test = PhysicalPointer::<u8>::new(madt_addr + size_of::<Madt>() + 1);
        serial_println!("record length: {}", *record_len_test);

        while current < end {
            serial_println!("current: 0x{:x}", current);
            let addr = current;
            current_entry = PhysicalPointer::new(addr);

            match current_entry.entry_type {
                0 => {
                    serial_println!("Processor Local APIC");
                    let proc_loc_apic: PhysicalPointer<ProcessorLocalApic> =
                        PhysicalPointer::new(addr);
                    terminal_println!("acpi proc id: {}", proc_loc_apic.acpi_processor_id);
                    terminal_println!("acpi id: {}", proc_loc_apic.acpi_id);
                    let flags = proc_loc_apic.flags;
                    terminal_println!("acpi flags: {}", flags);
                }
                1 => {
                    serial_println!("I/O APIC");
                    let io_apic = PhysicalPointer::<IoApic>::new(addr);
                    terminal_println!("io apic id: {}", io_apic.io_apic_id);
                    terminal_println!("reserved: {} ", io_apic.reserved);
                    let io_apic_addr = io_apic.io_apic_addr;
                    terminal_println!("io apic addr: 0x{:x} ", io_apic_addr);
                    terminal_println!("io apic addr as dec: {} ", io_apic_addr);
                    let gsib = io_apic.global_system_interrupt_base;
                    terminal_println!("gsib: {} ", gsib);

                    let io_apic: PhysicalPointer<IoApicRegister> =
                        PhysicalPointer::new(addr as usize);

                    terminal_println!("delivery mode: {}", io_apic.delivery_mode());
                }
                2 => {
                    serial_println!("I/O APIC Interrupt Source Override");
                    let ptr: PhysicalPointer<IoApicInterruptSourceOverride> =
                        PhysicalPointer::new(addr);
                    terminal_println!("bus source: {}", ptr.bus_source);
                    terminal_println!("irq source: {}", ptr.irq_source);
                    let gsi = ptr.global_system_interrupt;
                    terminal_println!("global system interrupt: {}", gsi);
                }
                3 => {
                    serial_println!("I/O APIC Non-maskable interrupt source");
                }
                4 => {
                    serial_println!("Local APIC Non-maskable interrupts");
                }
                5 => {
                    serial_println!("Local APIC Address Override");
                    let ptr = PhysicalPointer::<LocalApicOverride>::new(addr);

                    terminal_println!("id: {}", ptr.entry_type);
                    let local_apic_addr = ptr.local_apic_addr;
                    terminal_println!("addr: {}", local_apic_addr);
                }
                9 => {
                    serial_println!("Processor Local x2APIC");
                }
                _ => {
                    serial_println!("don't know");
                }
            }
            terminal_println!("record length: {}", current_entry.record_length);
            current += current_entry.record_length as usize;
        }
    }
}
fn find_signature(
    root: &PhysicalPointer<AcpiSdtHeader>,
    signature: &[u8; 4],
) -> Option<PhysicalPointer<AcpiSdtHeader>> {
    unsafe {
        let rsdt: PhysicalPointer<Rsdt> = PhysicalPointer::new(root.address() as usize);
        let rsdt = rsdt.read();
        terminal_println!("rsdt h: {}", from_utf8(&rsdt.h.signature[..]).unwrap());
        let rsdt_h = rsdt.h;
        terminal_println!(
            "rsdt signature: {}",
            from_utf8(&rsdt_h.signature[..]).unwrap()
        );
        terminal_println!("length: {}", rsdt_h.length);
        let entries = (rsdt_h.length as usize - size_of::<AcpiSdtHeader>()) / 4;
        terminal_println!("entries: {}", entries);
        for i in 0..entries {
            terminal_println!("begin ptr other: {}", i);
            terminal_println!("ptr other addr: {}", rsdt.pointer_to_other_sdt as usize);
            let ptr_other: PhysicalPointer<u32> = PhysicalPointer::new(
                (root.address() + size_of::<AcpiSdtHeader>() + (i * 4)) as usize,
            );
            terminal_println!("ptr other: {}", ptr_other.read());
            terminal_println!("begin phys ptr");
            let h: PhysicalPointer<AcpiSdtHeader> = PhysicalPointer::new(ptr_other.read() as usize);
            terminal_println!("signature: {}", from_utf8(&h.signature[..]).unwrap());
            if &(*h).signature == &signature[..] {
                return Some(h);
            }
        }
    }
    None
}

unsafe extern "C" {
    fn uacpi_for_each_subtable(//struct acpi_sdt_hdr *hdr, size_t hdr_size,
        //uacpi_subtable_iteration_callback cb, void *user
    );
}

fn cpuid(thing: u32) -> (u32, u32, u32, u32) {
    let eax;
    let ebx;
    let ecx;
    let edx;
    unsafe {
        core::arch::asm!(
            "push rbx",
            "cpuid",
            "mov {ebx:e}, ebx",
            "pop rbx",
            inout("eax") thing => eax,
            ebx = out(reg) ebx,
            out("ecx") ecx,
            out("edx") edx,

        );
    };
    (eax, ebx, ecx, edx)
}
