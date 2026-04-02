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
static HHDM_OFFSET: SyncUnsafeCell<u64> = SyncUnsafeCell::new(0);

pub fn get_hhdm_offset() -> u64 {
    unsafe { *HHDM_OFFSET.get() }
}
pub fn init_memory(entries: &[&Entry], hhdm_offset: u64) {
    unsafe {
        (*HHDM_OFFSET.get()) = hhdm_offset;
        serial_println!("hhdm_offset: {}", (*HHDM_OFFSET.get()));
    }
    let mut current = null_mut();
    for entry in entries {
        if entry.entry_type == EntryType::USABLE {
            serial_println!(
                "USABLE base: 0b{:064b}, length: {}",
                entry.base,
                entry.length
            );
            let mut curr_size = 0u64;
            while curr_size < entry.length {
                let region = ((entry.base + curr_size) + hhdm_offset) as *mut Region;
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
        (node as u64 - *(HHDM_OFFSET.get())) as *mut u8
    }
}

pub fn free(addr: *mut u8) {
    unsafe {
        let node = addr as *mut Region;
        (*REGION_HEAD.get()) = Region { next: node };
    }
}
