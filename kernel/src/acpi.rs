use core::str::from_utf8;

use crate::{memory::pmm::PhysicalPointer, serial_println};

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

pub fn init_acpi(rsdp_address: usize) {
    serial_println!("rsdp: {}", rsdp_address);

    serial_println!("thing start");
    let phys_ptr: PhysicalPointer<Rsdp> = PhysicalPointer::new(rsdp_address);

    let a = unsafe { phys_ptr.read() };
    serial_println!("thing signature: {}", from_utf8(&a.signature[..]).unwrap());
    serial_println!("thing checksum: {}", a.checksum);
    let rsdt_addr = a.rsdt_address;
    serial_println!("rsdt address: {}", rsdt_addr);

    let rsdt: PhysicalPointer<AcpiSdtHeader> = PhysicalPointer::new(rsdt_addr.try_into().unwrap());
    let rsdt_read = unsafe { rsdt.read() };
    serial_println!("length:{}", rsdt_read.length);
    serial_println!("revision:{}", rsdt_read.revision);
    serial_println!(
        "rsdt signature: {}",
        from_utf8(&rsdt_read.signature[..]).unwrap()
    );
    serial_println!("start finding signature");

    let a = unsafe { find_signature(b"MADT", &*(rsdt.vaddr as *const AcpiSdtHeader)) };
    if let Some(a) = a {
        serial_println!("found");
        serial_println!("{}", from_utf8(&a.signature[..]).unwrap());
    } else {
        serial_println!("sad :((");
    }
}

fn find_signature(signature: &'static [u8; 4], root: &AcpiSdtHeader) -> Option<AcpiSdtHeader> {
    let entries = root.length as usize - size_of::<AcpiSdtHeader>() / 4;
    serial_println!("entries: {}", entries);

    unsafe {
        let mut i = 0usize;

        while i < 4095 {
            let a = (root as *const AcpiSdtHeader as *const u8).add(i);

            serial_println!("{}", *a);
            i += 1;
        }
        for i in 0..entries {
            let h = (root as *const AcpiSdtHeader).add(i * size_of::<AcpiSdtHeader>());

            //serial_println!("{}", (from_utf8(&(&*h).signature[..]).unwrap()));
            if (*h).signature == *signature {
                return Some(*h);
            }
        }
    }
    None
}
