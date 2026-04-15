use alloc::boxed::Box;
use spin::Mutex;

use crate::{
    idt::{disable_interrupts, enable_interrupts},
    thread::ThreadControlBlock,
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
        if self.current.is_none() {
            self.current = Some(self.head);
        } else {
            if let Some(current) = self.current {
                if current.is_null() {
                    return;
                }
                unsafe {
                    let next = (*current).next;
                    if !next.is_null() {
                        self.current = Some(next);
                        switch(&*current, &*next);
                    }
                }
            }
        }
    }
}
#[unsafe(naked)]
pub unsafe extern "C" fn switch(current: &ThreadControlBlock, next: &ThreadControlBlock) {
    core::arch::naked_asm!(
        "
        push rax
        push rbx
        push rcx
        push rdx
        push rsi
        push rdi
        push rsp
        push rbp
        push r8
        push r9
        push r10
        push r11
        push r12
        push r13
        push r14
        push r15

        mov [rdi + {rax_offset}], rax
        mov [rdi + {rbx_offset}], rbx
        mov [rdi + {rcx_offset}], rcx
        mov [rdi + {rdx_offset}], rdx
        mov [rdi + {rsi_offset}], rsi
        mov [rdi + {rdi_offset}], rdi
        mov [rdi + {rbp_offset}], rbp
        mov [rdi + {r8_offset}], r8
        mov [rdi + {r9_offset}], r9
        mov [rdi + {r10_offset}], r10
        mov [rdi + {r11_offset}], r11
        mov [rdi + {r12_offset}], r12
        mov [rdi + {r13_offset}], r13
        mov [rdi + {r14_offset}], r14
        mov [rdi + {r15_offset}], r15
        mov [rdi + {rsp_offset}], rsp


        mov rsp, [rsi + {rsp_offset}]
        mov rax, [rsi + {rax_offset}]
        mov rbx, [rsi + {rbx_offset}]
        mov rcx, [rsi + {rcx_offset}]
        mov rdx, [rsi + {rdx_offset}]
        mov rbp, [rsi + {rbp_offset}]
        mov r8, [rsi + {r8_offset}]
        mov r9, [rsi + {r9_offset}]
        mov r10, [rsi + {r10_offset}]
        mov r11, [rsi + {r11_offset}]
        mov r12, [rsi + {r12_offset}]
        mov r13, [rsi + {r13_offset}]
        mov r14, [rsi + {r14_offset}]
        mov r15, [rsi + {r15_offset}]
        mov rdi, [rsi + {rdi_offset}]
        pop r15
        pop r14
        pop r13
        pop r12
        pop r11
        pop r10
        pop r9
        pop r8
        pop rbp
        pop rsp
        pop rdi
        pop rsi
        pop rdx
        pop rcx
        pop rbx
        pop rax
        ret
        ",
        rax_offset = const offset_of!(Registers, rax),
        rbx_offset = const offset_of!(Registers, rbx),
        rcx_offset = const offset_of!(Registers, rcx),
        rdx_offset = const offset_of!(Registers, rdx),
        rsi_offset = const offset_of!(Registers, rsi),
        rdi_offset = const offset_of!(Registers, rdi),
        rsp_offset = const offset_of!(Registers, rsp),
        rbp_offset = const offset_of!(Registers, rbp),
        r8_offset = const offset_of!(Registers, r8),
        r9_offset = const offset_of!(Registers, r9),
        r10_offset = const offset_of!(Registers, r10),
        r11_offset = const offset_of!(Registers, r11),
        r12_offset = const offset_of!(Registers, r12),
        r13_offset = const offset_of!(Registers, r13),
        r14_offset = const offset_of!(Registers, r14),
        r15_offset = const offset_of!(Registers, r15),
    )
}
pub fn init_multitasking() {}
pub fn init_scheduler() {
    disable_interrupts();
    for i in 0..10 {
        let node = Box::into_raw(Box::new(ThreadControlBlock::new(i as u64)));
        add_process(node);
    }
    enable_interrupts();
}

fn add_process(node: *mut ThreadControlBlock) {
    let mut scheduler = SCHEDULER.lock();

    if scheduler.head.is_null() {
        scheduler.head = node;
        unsafe {
            (*node).next = scheduler.head;
        }
        scheduler.head = node;
        scheduler.tail = Some(node);
    } else {
        unsafe {
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

#[unsafe(naked)]
pub unsafe extern "C" fn apply_registers(register: &Registers) {
    core::arch::naked_asm!(
        r#"
        mov rax, [rdi + {rax_offset}]
        mov rbx, [rdi + {rbx_offset}]
        mov rcx, [rdi + {rcx_offset}]
        mov rdx, [rdi + {rdx_offset}]
        mov rsi, [rdi + {rsi_offset}]
        mov rsp, [rdi + {rsp_offset}]
        mov rbp, [rdi + {rbp_offset}]
        mov r8, [rdi + {r8_offset}]
        mov r9, [rdi + {r9_offset}]
        mov r10, [rdi + {r10_offset}]
        mov r11, [rdi + {r11_offset}]
        mov r12, [rdi + {r12_offset}]
        mov r13, [rdi + {r13_offset}]
        mov r14, [rdi + {r14_offset}]
        mov r15, [rdi + {r15_offset}]
        mov rdi, [rdi + {rdi_offset}]
        iretq
        "#,
        rax_offset = const offset_of!(Registers, rax),
        rbx_offset = const offset_of!(Registers, rbx),
        rcx_offset = const offset_of!(Registers, rcx),
        rdx_offset = const offset_of!(Registers, rdx),
        rsi_offset = const offset_of!(Registers, rsi),
        rdi_offset = const offset_of!(Registers, rdi),
        rsp_offset = const offset_of!(Registers, rsp),
        rbp_offset = const offset_of!(Registers, rbp),
        r8_offset = const offset_of!(Registers, r8),
        r9_offset = const offset_of!(Registers, r9),
        r10_offset = const offset_of!(Registers, r10),
        r11_offset = const offset_of!(Registers, r11),
        r12_offset = const offset_of!(Registers, r12),
        r13_offset = const offset_of!(Registers, r13),
        r14_offset = const offset_of!(Registers, r14),
        r15_offset = const offset_of!(Registers, r15),
    );
}

#[unsafe(naked)]
pub unsafe extern "C" fn save_registers(register: &mut Registers) {
    core::arch::naked_asm!(
        r#"
        mov [rdi + {rax_offset}], rax
        mov [rdi + {rbx_offset}], rbx
        mov [rdi + {rcx_offset}], rcx
        mov [rdi + {rdx_offset}], rdx
        mov [rdi + {rsi_offset}], rsi
        mov [rdi + {rsp_offset}], rsp
        mov [rdi + {rbp_offset}], rbp
        mov [rdi + {r8_offset}], r8
        mov [rdi + {r9_offset}], r9
        mov [rdi + {r10_offset}], r10
        mov [rdi + {r11_offset}], r11
        mov [rdi + {r12_offset}], r12
        mov [rdi + {r13_offset}], r13
        mov [rdi + {r14_offset}], r14
        mov [rdi + {r15_offset}], r15
        mov [rdi + {rdi_offset}], rdi
        ret
        "#,
        rax_offset = const offset_of!(Registers, rax),
        rbx_offset = const offset_of!(Registers, rbx),
        rcx_offset = const offset_of!(Registers, rcx),
        rdx_offset = const offset_of!(Registers, rdx),
        rsi_offset = const offset_of!(Registers, rsi),
        rdi_offset = const offset_of!(Registers, rdi),
        rsp_offset = const offset_of!(Registers, rsp),
        rbp_offset = const offset_of!(Registers, rbp),
        r8_offset = const offset_of!(Registers, r8),
        r9_offset = const offset_of!(Registers, r9),
        r10_offset = const offset_of!(Registers, r10),
        r11_offset = const offset_of!(Registers, r11),
        r12_offset = const offset_of!(Registers, r12),
        r13_offset = const offset_of!(Registers, r13),
        r14_offset = const offset_of!(Registers, r14),
        r15_offset = const offset_of!(Registers, r15),
    );
}
