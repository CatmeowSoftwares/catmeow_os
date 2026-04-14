use core::{hint::spin_loop, ptr::null_mut, sync::atomic::AtomicU64};

use alloc::boxed::Box;

use crate::{
    memory::{
        PAGE_SIZE,
        pmm::get_hhdm_offset,
        vmm::{PAGE_MAP_LEVEL_4, PageMapLevel4, PageTableEntry, allocate, allocate_page},
    },
    scheduler::{Registers, SCHEDULER},
    terminal_println,
};

unsafe impl Sync for Process {}
pub struct Process {
    pub(crate) next: *mut Process,
}
#[derive(Default)]
pub(crate) struct ProcessControlBlock {
    pub(crate) id: u64,
    pub(crate) rsp: u64,
    pub(crate) cr3: u64,
    pub(crate) next: *mut ProcessControlBlock,
    pub(crate) state: u8,
    pub(crate) registers: Registers,
    pub(crate) time_used: u64,
}
fn process_entry() -> ! {
    let scheduler = SCHEDULER.lock();
    if let Some(current) = scheduler.current {
        let id = unsafe { (*current).id };
        terminal_println!("running: {}", id);
    }
    loop {
        spin_loop();
    }
}
impl ProcessControlBlock {
    pub(crate) fn new(id: u64) -> Self {
        let mut pcb = Self {
            id,
            rsp: 0,
            cr3: 0,
            next: null_mut(),
            state: 0,
            time_used: 0,
            ..Default::default()
        };
        let mut page_table_entry = PageTableEntry::new();
        page_table_entry.set_writable(true);
        let page = allocate_page(PAGE_SIZE as usize, 3);
        let stack_top = page + PAGE_SIZE;
        unsafe {
            *((stack_top - 8) as *mut u64) = 0;
            *((stack_top - 16) as *mut u64) = process_entry as *const u64 as u64;
        }
        pcb.registers.rsp = stack_top - 16;

        let pml4 = Box::into_raw(Box::new(PageMapLevel4([page_table_entry; 512])));

        pcb
    }
}
