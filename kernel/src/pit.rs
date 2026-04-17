use core::sync::atomic::AtomicU32;

use crate::{
    gui::put_line,
    idt::{disable_interrupts, enable_interrupts},
    scheduler::{self, SCHEDULER},
    serial::{inb, outb},
    serial_println, terminal_print, terminal_println,
};

const PIT_FREQUENCY: u32 = 1193182;
static COUNT_DOWN: AtomicU32 = AtomicU32::new(0);
pub fn remap_pic() {
    unsafe {
        let a1 = inb(0x21);
        let a2 = inb(0xA1);
        outb(0x20, 0x11);
        outb(0xA0, 0x11);
        outb(0x21, 0x20);
        outb(0xA1, 0x28);
        outb(0x21, 0x04);
        outb(0xA1, 0x02);
        outb(0x21, 0x01);
        outb(0xA1, 0x01);
        outb(0x21, a1);
        outb(0xA1, a2);
    }
}
pub fn init_pit() {
    disable_interrupts();
    remap_pic();
    unsafe {
        let current_mask = inb(0x21);
        outb(0x21, current_mask & 0xFE);
        outb(0x43, 0x36);
    }
    //sleep(10000);
    let divisor = (PIT_FREQUENCY / 1000) as u16;
    set_pit_count(divisor);
    enable_interrupts();
}
static MS_TICK: AtomicU32 = AtomicU32::new(0);
pub(crate) unsafe extern "x86-interrupt" fn timer_interrupt_handler(
    _stack_frame: crate::idt::InterruptStackFrame,
) {
    MS_TICK.fetch_add(1, core::sync::atomic::Ordering::Relaxed);
    COUNT_DOWN
        .try_update(
            core::sync::atomic::Ordering::Relaxed,
            core::sync::atomic::Ordering::Relaxed,
            |a| Some(a.saturating_sub(1)),
        )
        .ok();
    unsafe {
        outb(0x20, 0x20);
    }
    if let Some(mut scheduler) = SCHEDULER.try_lock() {
        scheduler.schedule();
    }
}
pub fn get_ms() -> u32 {
    MS_TICK.load(core::sync::atomic::Ordering::Relaxed)
}
pub fn wait(ms: u32) {
    let start = get_ms();
    let end = start.wrapping_add(ms);
    if end >= start {
        while get_ms() < end {}
    } else {
        while get_ms() >= start || get_ms() < end {}
    }
}

pub fn sleep(ms: u32) {
    COUNT_DOWN.store(ms, core::sync::atomic::Ordering::Relaxed);
    while COUNT_DOWN.load(core::sync::atomic::Ordering::Relaxed) > 0 {
        unsafe {
            core::arch::asm!("sti", "hlt", "cli");
        }
    }
    enable_interrupts();

    serial_println!("done")
}
fn read_pit_count() -> u16 {
    let mut count: u16;
    disable_interrupts();
    unsafe {
        outb(0x43, 0b0000000);
        count = inb(0x40) as u16;
        count |= ((inb(0x40) as u32) << 8) as u16;
    }
    enable_interrupts();
    count
}

fn set_pit_count(count: u16) {
    unsafe {
        outb(0x40, count as u8 & 0xFF);
        outb(0x40, ((count & 0xFF00) >> 8) as u8);
    }
}
