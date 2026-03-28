//bitmap allocator
// n/8 bytes

use crate::serial_println;
use core::{
    cell::SyncUnsafeCell,
    ptr::{eq, null_mut},
};
use limine::memory_map::{Entry, EntryType};
const PAGE_SIZE: u64 = 4096;

struct Region {
    next: *mut Region,
}
unsafe impl Send for Region {}
unsafe impl Sync for Region {}
static REGION_HEAD: SyncUnsafeCell<Region> = SyncUnsafeCell::new(Region { next: null_mut() });
pub fn init_memory(entries: &[&Entry], offset: u64) {
    serial_println!("start of entry forloop");
    for entry in entries {
        if entry.entry_type == EntryType::USABLE {
            serial_println!("USABLE base: 0x{:x}, length: {}", entry.base, entry.length);
            let mut current = REGION_HEAD.get();
            let mut curr_size = 0u64;
            while curr_size < entry.length {
                let region = ((entry.base + curr_size) + offset) as *mut Region;
                unsafe {
                    if (*current).next.is_null() {
                        (*current).next = region;
                    }
                    current = (*current).next;
                    curr_size += PAGE_SIZE;
                }
            }

            serial_println!("1");
        } else if entry.entry_type == EntryType::RESERVED {
            serial_println!(
                "RESERVED base: 0x{:x}, length: {}",
                entry.base,
                entry.length
            );
        }
        serial_println!("2");
    }
    serial_println!("end of forloop entry");

    serial_println!("starting printing data in the linked list");
    unsafe {
        let mut current = REGION_HEAD.get();
        let mut count: usize = 0;
        loop {
            if (*current).next.is_null() {
                break;
            } else {
                current = (*current).next;
                count += 1;
            }
        }
        serial_println!("we have count of: {}", count);
    }
    let a = allocate();
    serial_println!("a is: {:p}", a);
    free(a);
    serial_println!("a gone?: {:p}", a);
}

pub fn allocate() -> *mut u8 {
    let region_head = REGION_HEAD.get();
    let mut current = region_head;
    loop {
        unsafe {
            if (*current).next.is_null() {
                break;
            }
            current = (*current).next;
        }
    }
    current as *mut u8
}

pub fn free(addr: *mut u8) {
    let region = addr as *mut Region;
    let region_head = REGION_HEAD.get();
    let mut current = region_head;
    let mut previous = region_head;

    loop {
        unsafe {
            if (*current).next.is_null() {
                serial_println!("sad no more");
                break;
            }
            if eq(current, region) {
                serial_println!("ptr dies (sad)");
                (*previous).next = null_mut();
                break;
            }
            previous = current;
            current = (*current).next;
        }
    }
}
