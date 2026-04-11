use alloc::boxed::Box;

use crate::{
    gui::{put_pixel, put_rect},
    process::{Process, ProcessControlBlock},
    serial_println, terminal_println,
    tsc::get_ms,
};
use core::{cell::SyncUnsafeCell, mem::offset_of, ptr::null_mut, sync::atomic::AtomicU64};
static PROCESS_HEAD: SyncUnsafeCell<Process> = SyncUnsafeCell::new(Process { next: null_mut() });

static SCHEDULER: SyncUnsafeCell<Scheduler> = SyncUnsafeCell::new(Scheduler::new());
unsafe impl Sync for Scheduler {}
struct Scheduler {
    head: ProcessControlBlock,
    tail: Option<*mut ProcessControlBlock>,
    current: Option<&'static mut ProcessControlBlock>,
    next: Option<&'static mut ProcessControlBlock>,
}

impl Scheduler {
    const fn new() -> Self {
        Self {
            tail: None,
            head: ProcessControlBlock::new(0),
            current: None,
            next: None,
        }
    }
}
static CURRENT_COUNT: AtomicU64 = AtomicU64::new(0);
static LAST_COUNT: AtomicU64 = AtomicU64::new(0);
pub fn init_multitasking() {}
pub fn init_scheduler() {
    let scheduler = unsafe { &mut *SCHEDULER.get() };
    for i in 0..10 {
        let node = Box::into_raw(Box::new(ProcessControlBlock::new(i as u64)));
        add_process(node);
    }
    let time_to_wait = 1000; //seconds
    let mut current_time;
    let mut last_schedule_time = 0;
    let register = Registers::default();
    serial_println!("offset of! {}", offset_of!(Registers, rax));
    serial_println!("offset of! {}", offset_of!(Registers, rbx));
    serial_println!("offset of! {}", offset_of!(Registers, cr3));
    loop {
        put_rect(21, 21, 500, 500, 0xFF00FF);
        current_time = get_ms();
        //serial_println!("current time: {}", current_time);
        if current_time - last_schedule_time >= time_to_wait {
            last_schedule_time = current_time;
            schedule();
        }
    }
}

fn schedule() {
    let scheduler = unsafe { &mut *SCHEDULER.get() };
    if scheduler.current.is_none() {
        scheduler.current = unsafe { scheduler.head.next.as_mut() };
    } else {
        let next_ptr = scheduler.current.as_ref().map(|c| c.next);
        if let Some(ptr) = next_ptr {
            unsafe {
                scheduler.current = ptr.as_mut();
            }
        }
    }
    if let Some(ref current) = scheduler.current {
        terminal_println!("current PID: {}", current.id);
        if let Some(next) = unsafe { current.next.as_mut() } {
            terminal_println!("next PID: {}", next.id);
        }
    }
}

fn add_process(node: *mut ProcessControlBlock) {
    let scheduler = unsafe { &mut *SCHEDULER.get() };

    if scheduler.head.next.is_null() {
        unsafe {
            (*node).next = &raw mut scheduler.head;
        }
        scheduler.head.next = node;
        scheduler.tail = Some(node);
    } else {
        unsafe {
            let old_tail = scheduler.tail.unwrap();
            (*old_tail).next = node;
            (*node).next = &raw mut scheduler.head;
            scheduler.tail = Some(node);
        }
    }
}
fn remove_process() {}

fn context_switch() {
    let rsp: u64;
    unsafe {
        core::arch::asm!("mov {rsp}, rsp", rsp = out(reg) rsp);
    }
}

#[derive(Default)]
struct Registers {
    rax: u64,
    rbx: u64,
    rcx: u64,
    rdx: u64,
    rsi: u64,
    rdi: u64,
    rsp: u64,
    rbp: u64,
    r8: u64,
    r9: u64,
    r10: u64,
    r11: u64,
    r12: u64,
    r13: u64,
    r14: u64,
    r15: u64,
    cr3: u64,
}
impl Registers {
    fn new() -> Self {
        Self::default()
    }
}
#[unsafe(naked)]
pub unsafe extern "C" fn test(register: &Registers) {
    core::arch::naked_asm!("");
}
fn save_current_regs() -> Registers {
    let mut register = Registers::new();
    register
}
