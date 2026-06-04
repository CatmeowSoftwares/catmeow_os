use alloc::boxed::Box;
use spin::Mutex;

use crate::{
    idt::{disable_interrupts, enable_interrupts},
    serial_println, terminal_print, terminal_println,
    thread::{ThreadControlBlock, ThreadStatus},
};
use core::{mem::offset_of, ptr::null_mut};

pub static SCHEDULER: Mutex<Scheduler> = Mutex::new(Scheduler::new());
unsafe impl Send for Scheduler {}
unsafe impl Sync for Scheduler {}
pub struct Scheduler {
    head: *mut ThreadControlBlock,
    tail: Option<*mut ThreadControlBlock>,
    pub(crate) current: Option<*mut ThreadControlBlock>,
    next: Option<*mut ThreadControlBlock>,
}
impl Scheduler {
    const fn new() -> Self {
        Self {
            tail: None,
            head: null_mut(),
            current: None,
            next: None,
        }
    }
    pub fn schedule(&mut self) {
        if self.head.is_null() {
            return;
        }
        serial_println!("c");
        if self.current.is_none() {
            let first = self.head;
            unsafe { (*first).status = ThreadStatus::Running };
            self.current = Some(first);
        } else {
            if let Some(current) = self.current {
                if current.is_null() {
                    return;
                }
                unsafe {
                    let mut next = (*current).next;
                    let start = next;
                    /*
                    loop {
                        if (*next).status != ThreadStatus::Ready {
                            break;
                        }
                        next = (*next).next;
                        if next == start {
                            return;
                        }
                    }
                    */
                    (*current).status = ThreadStatus::Ready;
                    (*next).status = ThreadStatus::Running;
                    switch((*current).registers.rsp, (*next).registers.rsp);
                    self.current = Some(next);
                    serial_println!("s!");
                    serial_println!("c: {}!", (*current).id);
                }
            }
        }
    }
}
#[unsafe(naked)]
pub unsafe extern "C" fn switch(current: u64, next: u64) {
    core::arch::naked_asm!(
        "
        cli
        push r15
        push r14
        push r13
        push r12
        push r11
        push r10
        push r9
        push r8
        push rbp
        push rdi
        push rsi
        push rdx
        push rcx
        push rbx
        push rax

        #mov rax,cr3
        #push rax

        mov [rdi], rsp
        mov rsp, [rsi]

        #pop rax
        #mov cr3, rax

        pop rax
        pop rbx
        pop rcx
        pop rdx
        pop rsi
        pop rdi
        pop rbp
        pop r8
        pop r9
        pop r10
        pop r11
        pop r12
        pop r13
        pop r14
        pop r15
        #add rsp,16
        #sti
        #iretq
        ret
        ",
    )
}
#[unsafe(naked)]
pub unsafe extern "C" fn threat() {
    core::arch::naked_asm!(
        "

        "
    )
}
pub fn init_multitasking() {}
pub fn init_scheduler() {
    terminal_println!("schedulerr");
    disable_interrupts();
    for i in 0..10 {
        let node = Box::into_raw(Box::new(ThreadControlBlock::new(i as u64)));
        add_process(node);
    }
    let sched = SCHEDULER.lock();
    let mut current = sched.head;
    unsafe {
        loop {
            terminal_println!("{}", (*current).id);
            current = (*current).next;
            if current == sched.head {
                break;
            }
        }
    }
}

fn add_process(node: *mut ThreadControlBlock) {
    let mut scheduler = SCHEDULER.lock();
    if scheduler.head.is_null() {
        scheduler.head = node;
        terminal_println!("gave head");
        unsafe {
            (*node).next = node;
        }
        scheduler.tail = Some(node);
    } else {
        unsafe {
            terminal_println!("AAAAAAAA");
            let old_tail = scheduler.tail.unwrap();
            (*old_tail).next = node;
            (*node).next = scheduler.head;
            scheduler.tail = Some(node);
        }
    }
}

#[derive(Default, Clone, Copy)]
pub struct Registers {
    pub rax: u64,
    pub rbx: u64,
    pub rcx: u64,
    pub rdx: u64,
    pub rsi: u64,
    pub rdi: u64,
    pub rsp: u64,
    pub rbp: u64,
    pub rip: u64,
    pub r8: u64,
    pub r9: u64,
    pub r10: u64,
    pub r11: u64,
    pub r12: u64,
    pub r13: u64,
    pub r14: u64,
    pub r15: u64,
}
