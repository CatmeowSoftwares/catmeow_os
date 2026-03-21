use core::{arch::asm, ptr::copy};

use alloc::boxed::Box;
use x86_64::{
    PhysAddr,
    registers::control::{Cr3, Cr3Flags},
    structures::paging::{PhysFrame, Size4KiB},
};

pub struct Register {
    rip: u64,
    rsp: u64,

    rax: u64,
    rbx: u64,
    rcx: u64,
    rdx: u64,
    rsi: u64,
    rbp: u64,
    r8: u64,
    r9: u64,
    r10: u64,
    r11: u64,
    r12: u64,
    r13: u64,
    r14: u64,
    r15: u64,

    cs: u64,
    ds: u64,
    ss: u64,
    es: u64,
    fs: u64,
    gs: u64,

    cr3: u64,
}

impl Register {
    fn new() -> Self {
        Self {
            rip: 0,
            rsp: 0,

            rax: 0,
            rbx: 0,
            rcx: 0,
            rdx: 0,
            rsi: 0,
            rbp: 0,
            r8: 0,
            r9: 0,
            r10: 0,
            r11: 0,
            r12: 0,
            r13: 0,
            r14: 0,
            r15: 0,

            cs: 0,
            ds: 0,
            ss: 0,
            es: 0,
            fs: 0,
            gs: 0,

            cr3: 0,
        }
    }
}
pub struct Process<'a> {
    registers: Register,
    cr3: PhysFrame,
    next: Option<&'a mut Process<'a>>,
}
pub struct Scheduler<'a> {
    pub queue: Option<&'a mut Process<'a>>,
    pub current_process: Option<&'a mut Process<'a>>,
}

impl<'a> Process<'a> {
    pub fn new() -> Self {
        Self {
            registers: Register::new(),
            cr3: unsafe { PhysFrame::from_start_address_unchecked(PhysAddr::new(0)) },
            next: None,
        }
    }
    pub fn kill() {}
}

impl<'a> Scheduler<'a> {
    pub fn new() -> Self {
        Self {
            queue: None,
            current_process: None,
        }
    }
    pub fn switch_task(&'a mut self, register: &mut Register) {
        let next = {
            let current_process = match self.current_process.as_deref_mut() {
                Some(a) => a,
                None => return,
            };

            unsafe {
                copy(
                    &raw const current_process.registers,
                    &raw mut *register,
                    size_of::<Register>(),
                );
            }
            current_process.next.take()
        };

        self.current_process = next.or_else(|| self.queue.take());

        let next_process = match self.current_process.as_deref_mut() {
            Some(a) => a,
            None => return,
        };
        unsafe {
            copy(
                &raw const *register,
                &raw mut next_process.registers,
                size_of::<Register>(),
            );
        }

        switch_page_directory(next_process);
    }
}

fn switch_page_directory(process: &mut Process) {
    unsafe {
        asm!("mov cr3, {}", in(reg) process.cr3.start_address().as_u64(), options(nostack, preserves_flags));
    }
}

pub fn load_bin(data: *mut u8, size: usize) {
    let process = Box::new(Process::new());
}
