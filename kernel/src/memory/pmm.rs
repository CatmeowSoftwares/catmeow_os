use super::PAGE_SIZE;
use crate::{
    memory::vmm::{self, map_page, unmap_page},
    terminal_println,
};
use core::{
    marker::PhantomData,
    mem::forget,
    ops::{Deref, DerefMut},
    ptr::null_mut,
    sync::atomic::AtomicU64,
};
use limine::memory_map::{Entry, EntryType};
use spin::mutex::Mutex;
struct Region {
    next: *mut Region,
}
unsafe impl Send for Region {}
unsafe impl Sync for Region {}
static REGION_HEAD: Mutex<Region> = Mutex::new(Region { next: null_mut() });
static HHDM_OFFSET: AtomicU64 = AtomicU64::new(0);
const PAGE_MASK: u64 = PAGE_SIZE - 1;
pub fn get_hhdm_offset() -> u64 {
    HHDM_OFFSET.load(core::sync::atomic::Ordering::Relaxed)
}
pub fn init_memory(entries: &[&Entry], hhdm_offset: u64) {
    HHDM_OFFSET.store(hhdm_offset, core::sync::atomic::Ordering::Relaxed);
    let mut current = null_mut();
    for entry in entries {
        match entry.entry_type {
            EntryType::USABLE => {
                let mut curr_size = 0u64;
                while curr_size < entry.length {
                    let region = ((entry.base + curr_size) + hhdm_offset) as *mut Region;
                    unsafe {
                        (*region).next = current;
                        current = region;
                        curr_size += PAGE_SIZE;
                    }
                }
            }
            EntryType::RESERVED => {}
            EntryType::ACPI_RECLAIMABLE => {}
            EntryType::ACPI_NVS => {}
            EntryType::BAD_MEMORY => {}
            EntryType::BOOTLOADER_RECLAIMABLE => {}
            EntryType::EXECUTABLE_AND_MODULES => {}
            EntryType::FRAMEBUFFER => {}
            _ => {}
        }
    }
    *REGION_HEAD.lock() = Region { next: current };
}

pub fn allocate() -> *mut u8 {
    unsafe {
        let mut head = REGION_HEAD.lock();

        let node = (*head).next;
        if node.is_null() {
            return null_mut();
        }

        head.next = (*node).next;
        (node as u64 - get_hhdm_offset()) as *mut u8
    }
}

pub fn free(addr: *mut u8) {
    unsafe {
        let mut head = REGION_HEAD.lock();
        let node = (addr as u64 + get_hhdm_offset()) as *mut Region;
        (*node).next = (*head).next;
        head.next = node;
    }
}

pub fn find(addr: *mut u8) -> Option<*mut u8> {
    terminal_println!("start of finding addr");
    let head = REGION_HEAD.lock();
    unsafe {
        let mut current = (*head).next;
        let hhdm_offset = get_hhdm_offset();
        while let Some(node) = (*current).next.as_ref() {
            let node_ptr = node as *const Region as *mut u8;
            terminal_println!(
                "ptr: 0x{:x}, rsdp: 0x{:x}, aligned rsdp: 0x{:x}",
                (node_ptr as u64 - hhdm_offset),
                addr as u64,
                page_align(addr as u64)
            );

            if (node_ptr as u64 - hhdm_offset) == page_align(addr as u64) {
                terminal_println!("found");
                return Some(node_ptr);
            }
            current = (*current).next;
        }
    }
    terminal_println!("0x{:x}", addr as u64);
    terminal_println!("0x{:x}", (addr as u64) & !4095);
    terminal_println!("sad none");
    None
}

pub const fn page_align(x: u64) -> u64 {
    x & !PAGE_MASK
}
pub const fn is_page_aligned(x: u64) -> bool {
    x == page_align(x)
}
