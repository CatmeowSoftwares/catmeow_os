use super::PAGE_SIZE;
use crate::{memory::pmm, serial_print, serial_println};
use bitfields::bitfield;

use core::{
    arch::{asm, global_asm},
    cell::SyncUnsafeCell,
    ptr::{null_mut, write_bytes},
};
#[bitfield(u64)]
struct VirtAddr {
    #[bits(12)]
    offset: u16,
    #[bits(9)]
    pt_index: u16,
    #[bits(9)]
    pd_index: u16,
    #[bits(9)]
    pdpt_index: u16,
    #[bits(9)]
    pml4_index: u16,
    #[bits(16)]
    sign_extension: u32,
}
impl VirtAddr {
    fn from_u64(n: u64) -> Self {
        Self(n)
    }
}
struct Node {
    next: *mut Node,
}
unsafe impl Sync for Node {}
unsafe impl Sync for VirtualAllocator {}
struct VirtualAllocator {
    head: Node,
}
impl VirtualAllocator {
    fn pop(&mut self) -> *mut u8 {
        unsafe {
            let head = &raw mut self.head;

            let node = (*head).next;
            if node.is_null() {
                return null_mut();
            }

            (*head).next = (*node).next;
            node as *mut u8
        }
    }
    fn push(&mut self, value: *mut u8) {
        unsafe {
            let node = value as *mut Node;
            (*node).next = self.head.next;
            self.head.next = node;
        }
    }
}
static VIRTUAL_ALLOCATOR: SyncUnsafeCell<VirtualAllocator> =
    SyncUnsafeCell::new(VirtualAllocator {
        head: Node { next: null_mut() },
    });
pub fn test() {
    let virtadr = VirtAddr::new();
    let mut c3: u64 = 0;
    unsafe {
        core::arch::asm!("mov {0:r}, cr3", out(reg) c3);
    }
    serial_println!("cr3: {}, bits: {:064b}", c3, c3);
}
pub fn allocate(start: u64, size: usize, flags: u64) {
    for i in 0..size / 4096 {
        let ptr = pmm::allocate();
        map_page(ptr, (start + i as u64 * PAGE_SIZE) as *mut u8, flags);
    }
}
pub fn allocate_with_ptr(start: u64, flags: u64) -> *mut u8 {
    let ptr = pmm::allocate();
    map_page(ptr, start as *mut u8, flags);
    serial_println!("mapping to virt: {}", start);
    ptr
}

pub fn free(addr: *mut u8, size: usize) {
    for i in 0..(size / PAGE_SIZE as usize) {
        pmm::free(get_physaddr(
            (addr as usize + i * PAGE_SIZE as usize) as *mut u8,
        ));
    }
}

#[repr(C, align(4096))]
struct PageMapLevel4([PageTableEntry; 512]);
#[repr(C, align(4096))]
struct PageDirectoryPointerTable([PageTableEntry; 512]);
#[repr(C, align(4096))]
struct PageDirectory([PageTableEntry; 512]);
#[repr(C, align(4096))]
struct PageTable([PageTableEntry; 512]);
#[bitfield(u64)]
#[derive(Clone, Copy)]
struct PageTableEntry {
    present: bool,
    writable: bool,
    user: bool,
    write_through: bool,
    cache_disabled: bool,
    accessed: bool,
    dirty: bool,
    page_size: bool,
    global: bool,
    #[bits(3)]
    available: u8,
    #[bits(40)]
    physical_address: u64,
    #[bits(7)]
    available_again: u8,
    #[bits(4)]
    protection_key: u8,
    no_execute: bool,
}

static PAGE_MAP_LEVEL_4: SyncUnsafeCell<PageMapLevel4> = {
    let mut page_table_entry = PageTableEntry::new();
    page_table_entry.set_writable(true);
    let pml4_table = PageMapLevel4([page_table_entry; 512]);
    SyncUnsafeCell::new(pml4_table)
};

pub fn init_vmm(phys_base: u64, virt_base: u64) {
    let kernel_offset = virt_base - phys_base;
    let test = u64::MAX;
    serial_println!("{:064b}", test & 0xffffffffff);

    let page_map_level_4 = unsafe { &mut (*PAGE_MAP_LEVEL_4.get()).0 };

    let hhdm = pmm::get_hhdm_offset();
    page_map_level_4[0].set_present(true);
    /*
       page_map_level_4[0]
           .set_physical_address((page_directory_pointer_table.as_ptr() as u64 - hhdm) >> 12);
       page_directory_pointer_table[0].set_present(true);
       page_directory_pointer_table[0]
           .set_physical_address((page_directory.as_ptr() as u64 - hhdm) >> 12);
       page_directory[0].set_present(true);
       page_directory[0].set_physical_address((page_table.as_ptr() as u64 - hhdm) >> 12);
    */
    let mut curr_cr3: u64;
    unsafe {
        asm!("mov {ptr}, cr3", ptr = out(reg) curr_cr3);
    }
    serial_println!("cr3: {}", curr_cr3);
    let lim_pml4 =
        unsafe { &*(((curr_cr3 & 0x000F_FFFF_FFFF_F000) + hhdm) as *const [PageTableEntry; 512]) };
    for i in 256..512 {
        page_map_level_4[i] = lim_pml4[i];
    }
    let ptr = (page_map_level_4.as_ptr() as u64) - kernel_offset;
    unsafe {
        asm!("
                mov cr3, {0:r}", in(reg) ptr);
    }
    serial_println!();
    serial_println!("paging enabled");
}
pub fn get_physaddr(virtaddr: *mut u8) -> *mut u8 {
    let page_map_level_4 = unsafe { &mut (*PAGE_MAP_LEVEL_4.get()).0 };
    let pml4_index = virtaddr as u64 >> 39 & 0x01FF;
    let pdpt_index = virtaddr as u64 >> 30 & 0x01FF;
    let pd_index = virtaddr as u64 >> 21 & 0x01FF;
    let pt_index = virtaddr as u64 >> 12 & 0x01FF;

    let hhdm = pmm::get_hhdm_offset();

    if !page_map_level_4[pml4_index as usize].present() {}
    let pdpt = ((page_map_level_4[pml4_index as usize].physical_address() << 12) + hhdm)
        as *mut PageTableEntry;
    let pdpt_entry = unsafe { &mut *pdpt.add(pdpt_index as usize) };
    if !pdpt_entry.present() {}

    let pd = ((pdpt_entry.physical_address() << 12) + hhdm) as *mut PageTableEntry;
    let pd_entry = unsafe { &mut *pd.add(pd_index as usize) };
    if !pd_entry.present() {}
    let pt = ((pd_entry.physical_address() << 12) + hhdm) as *mut PageTableEntry;
    let pt_entry = unsafe { &mut *pt.add(pt_index as usize) };
    pt_entry.physical_address() as *mut u8
}

pub fn map_page(physaddr: *mut u8, virtaddr: *mut u8, flags: u64) {
    let page_map_level_4 = unsafe { &mut (*PAGE_MAP_LEVEL_4.get()).0 };
    let pml4_index = virtaddr as u64 >> 39 & 0x01FF;
    let pdpt_index = virtaddr as u64 >> 30 & 0x01FF;
    let pd_index = virtaddr as u64 >> 21 & 0x01FF;
    let pt_index = virtaddr as u64 >> 12 & 0x01FF;
    let hhdm = pmm::get_hhdm_offset();

    if !page_map_level_4[pml4_index as usize].present() {
        let ptr = pmm::allocate();
        unsafe { write_bytes((ptr as u64 + hhdm) as *mut u8, 0, PAGE_SIZE as usize) };
        page_map_level_4[pml4_index as usize].set_present(true);
        page_map_level_4[pml4_index as usize].set_writable(true);
        page_map_level_4[pml4_index as usize].set_physical_address(ptr as u64 >> 12);
    }
    let pdpt = ((page_map_level_4[pml4_index as usize].physical_address() << 12) + hhdm)
        as *mut PageTableEntry;
    let pdpt_entry = unsafe { &mut *pdpt.add(pdpt_index as usize) };
    if !pdpt_entry.present() {
        let ptr = pmm::allocate();
        unsafe { write_bytes((ptr as u64 + hhdm) as *mut u8, 0, PAGE_SIZE as usize) };
        pdpt_entry.set_present(true);
        pdpt_entry.set_writable(true);
        pdpt_entry.set_physical_address(ptr as u64 >> 12);
    }

    let pd = ((pdpt_entry.physical_address() << 12) + hhdm) as *mut PageTableEntry;
    let pd_entry = unsafe { &mut *pd.add(pd_index as usize) };
    if !pd_entry.present() {
        let ptr = pmm::allocate();
        unsafe { write_bytes((ptr as u64 + hhdm) as *mut u8, 0, PAGE_SIZE as usize) };
        pd_entry.set_present(true);
        pd_entry.set_writable(true);
        pd_entry.set_physical_address(ptr as u64 >> 12);
    }
    let pt = ((pd_entry.physical_address() << 12) + hhdm) as *mut PageTableEntry;
    let pt_entry = unsafe { &mut *pt.add(pt_index as usize) };
    pt_entry.set_present(true);
    pt_entry.set_writable(flags & 1 != 0);
    pt_entry.set_physical_address(physaddr as u64 >> 12);
}

pub fn unmap_page(virtaddr: *mut u8) {
    let page_map_level_4 = unsafe { &mut (*PAGE_MAP_LEVEL_4.get()).0 };
    let pml4_index = virtaddr as u64 >> 39 & 0x01FF;
    let pdpt_index = virtaddr as u64 >> 30 & 0x01FF;
    let pd_index = virtaddr as u64 >> 21 & 0x01FF;
    let pt_index = virtaddr as u64 >> 12 & 0x01FF;
    let hhdm = pmm::get_hhdm_offset();
    let pdpt = ((page_map_level_4[pml4_index as usize].physical_address() << 12) + hhdm)
        as *mut PageTableEntry;
    let pdpt_entry = unsafe { &mut *pdpt.add(pdpt_index as usize) };
    let pd = ((pdpt_entry.physical_address() << 12) + hhdm) as *mut PageTableEntry;
    let pd_entry = unsafe { &mut *pd.add(pd_index as usize) };
    let pt = ((pd_entry.physical_address() << 12) + hhdm) as *mut PageTableEntry;
    let pt_entry = unsafe { &mut *pt.add(pt_index as usize) };
    pt_entry.set_bits(0);

    unsafe {
        asm!("
            invlpg [{0}]
            ",
            in(reg) virtaddr,
            options(nostack, preserves_flags)
        );
    }
}

struct Page {
    addr: u64,
}

impl Page {
    fn new() -> Self {
        Self { addr: 0 }
    }
    fn map(&self, phys: usize, virt: usize, flags: u64) {}
    fn unmap(&self) {}
    fn remap(&self) {}
}
