use core::ptr::null_mut;

use alloc::boxed::Box;

use crate::{
    memory::PAGE_SIZE, process::ProcessControlBlock, scheduler::Registers, terminal_println,
};

#[derive(Default)]
pub struct Thread {
    id: u64,
    registers: Registers,
    pc: u64,
    pcb_ptr: *mut ProcessControlBlock,
}

impl Thread {
    fn new(id: u64) -> Self {
        Self {
            id,
            ..Default::default()
        }
    }
}

fn create_thread() {
    let thread = Thread::new(0);
}
#[derive(Default)]
pub struct ThreadControlBlock {
    pub id: u64,
    pub next: *mut ThreadControlBlock,
    pub registers: Registers,
    pub cr3: u64,
    pub esp0: u64,
    pub state: u8,
}
unsafe impl Sync for ThreadControlBlock {}

fn meow() -> ! {
    loop {
        terminal_println!("meow :3");
    }
}
impl ThreadControlBlock {
    pub(crate) fn new(id: u64) -> Self {
        let tcb = Self {
            id,
            ..Default::default()
        };
        tcb
    }
}
