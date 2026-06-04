use core::{
    default,
    ptr::{null_mut, write_bytes},
};

use alloc::boxed::Box;

use crate::{
    memory::{
        PAGE_SIZE,
        pmm::{self, get_hhdm_offset},
        vmm,
    },
    process::ProcessControlBlock,
    scheduler::Registers,
    terminal_println,
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

#[derive(Default, PartialEq, Eq)]
pub enum ThreadStatus {
    #[default]
    New,
    Idle,
    Ready,
    Running,
    Blocked,
}
pub struct ThreadControlBlock {
    pub id: u64,
    pub next: *mut ThreadControlBlock,
    pub registers: Registers,
    pub cr3: u64,
    pub esp0: u64,
    pub rip: u64,
    pub status: ThreadStatus,
    pub function: fn(),
}
impl Default for ThreadControlBlock {
    fn default() -> Self {
        Self {
            id: 0,
            next: null_mut(),
            registers: Registers::default(),
            cr3: 0,
            esp0: 0,
            rip: 0,
            status: ThreadStatus::default(),
            function: meow,
        }
    }
}
unsafe impl Sync for ThreadControlBlock {}

fn meow() {
    loop {
        terminal_println!("meow :3");
        if let Some(scheduler) = crate::scheduler::SCHEDULER.try_lock() {
            terminal_println!("from: {:?}", scheduler.current);
        }
    }
}
impl ThreadControlBlock {
    pub(crate) fn new(id: u64) -> Self {
        let mut tcb = Self {
            id,
            ..Default::default()
        };
        let ptr = vmm::allocate_page(3) as u64;
        let stack_top = ptr + PAGE_SIZE;
        tcb.registers.rsp = stack_top;

        unsafe {
            core::arch::asm!("mov {}, cr3", out(reg) tcb.cr3);
        }

        tcb.status = ThreadStatus::Ready;
        tcb
    }
}
const _: u64 = 0xdeadbeef;
pub const THREAD_SIZE: usize = PAGE_SIZE as usize * 2;
