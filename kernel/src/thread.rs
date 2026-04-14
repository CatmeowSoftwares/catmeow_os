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

pub struct ThreadControlBlock {
    pub id: u64,
    pub next: *mut ThreadControlBlock,
    pub registers: Registers,
    pub cr3: u64,
    pub esp0: u64,
    pub stack: Box<[u8; PAGE_SIZE as usize]>,
}
impl Default for ThreadControlBlock {
    fn default() -> Self {
        Self {
            id: 0,
            next: null_mut(),
            registers: Registers::default(),
            cr3: 0,
            esp0: 0,
            stack: Box::new([0u8; PAGE_SIZE as usize]),
        }
    }
}
unsafe impl Sync for ThreadControlBlock {}

fn meow() -> ! {
    loop {
        terminal_println!("meow :3");
    }
}
impl ThreadControlBlock {
    pub(crate) fn new(id: u64) -> Self {
        let mut tcb = Self {
            id,
            ..Default::default()
        };
        let stack = Box::new([0u8; PAGE_SIZE as usize]);
        let top = stack.as_ptr() as u64 + PAGE_SIZE as u64;
        let stack_top = top - 8;
        unsafe {
            *(stack_top as *mut u64) = meow as *mut u64 as u64;
        }
        tcb.registers.rsp = stack_top;
        tcb.stack = stack;
        tcb
    }
}
