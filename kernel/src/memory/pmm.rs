use crate::serial_println;
use core::{cell::SyncUnsafeCell, ptr::null_mut};
use limine::memory_map::{Entry, EntryType};
const PAGE_SIZE: u64 = 4096;

struct Region {
    next: *mut Region,
}
unsafe impl Send for Region {}
unsafe impl Sync for Region {}
static REGION_HEAD: SyncUnsafeCell<Region> = SyncUnsafeCell::new(Region { next: null_mut() });
pub fn init_memory(entries: &[&Entry], offset: u64) {
    let mut current = null_mut();
    for entry in entries {
        if entry.entry_type == EntryType::USABLE {
            serial_println!("USABLE base: 0x{:x}, length: {}", entry.base, entry.length);

            let mut curr_size = 0u64;
            while curr_size < entry.length {
                let region = ((entry.base + curr_size) + offset) as *mut Region;
                unsafe {
                    (*region).next = current;
                    current = region;
                    curr_size += PAGE_SIZE;
                }
            }
        } else if entry.entry_type == EntryType::RESERVED {
            serial_println!(
                "RESERVED base: 0x{:x}, length: {}",
                entry.base,
                entry.length
            );
        }
    }
    unsafe { (*REGION_HEAD.get()) = Region { next: current } }
}

pub fn allocate() -> *mut u8 {
    unsafe {
        let head = REGION_HEAD.get();

        let node = (*head).next;
        if node.is_null() {
            return null_mut();
        }

        (*head).next = (*node).next;
        node as *mut u8
    }
}

pub fn free(addr: *mut u8) {
    unsafe {
        let node = addr as *mut Region;
        (*REGION_HEAD.get()) = Region { next: node };
    }
}
