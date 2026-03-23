use core::{
    arch::{self, asm},
    ptr::{copy, copy_nonoverlapping},
};

use alloc::boxed::Box;
use x86_64::{
    PhysAddr,
    registers::control::{Cr3, Cr3Flags},
    structures::paging::{FrameAllocator, Mapper, PhysFrame, Size4KiB},
};

use crate::{serial_println, terminal_println, vmm::alloc_page};

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

pub fn load_bin(
    data: *mut u8,
    size: usize,
    scheduler: &mut Scheduler,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
    mapper: &mut impl Mapper<Size4KiB>,
) {
    let process = Box::new(Process::new());

    scheduler.current_process = Some(&mut *Box::leak(process));

    let entry_addr = 0x400000usize;

    match alloc_page(entry_addr as u64, size as u64, frame_allocator, mapper) {
        Ok(v) => {
            serial_println!("{:?}", v);
        }
        Err(err) => {
            serial_println!("{:?}", err);
            return;
        }
    }
    unsafe {
        copy_nonoverlapping(data, entry_addr as *mut u8, size);
    }

    serial_println!("entry = {:x}", entry_addr);
    for i in 0..16 {
        unsafe {
            serial_println!("byte[{}] = {:02x}", i, *data.add(i));
        }
    }

    let entry: extern "C" fn() = unsafe { core::mem::transmute(entry_addr as usize) };

    //entry();
    serial_println!("test");
}
