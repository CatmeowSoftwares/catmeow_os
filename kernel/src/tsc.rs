use crate::{idt::enable_interrupts, pit::wait, serial_println};
use core::sync::atomic::AtomicU64;
pub fn init_tsc() {
    calibrate_tsc();
    sleep_ms(10_000);
}

fn rdtsc() -> (u32, u32) {
    let top: u32;
    let bottom: u32;
    unsafe {
        core::arch::asm!("rdtsc",
            out("edx") top,
            out("eax") bottom );
    }
    (top, bottom)
}
static TSC_HZ: AtomicU64 = AtomicU64::new(0);
fn tsc() -> u64 {
    let (top, bottom) = rdtsc();
    (top as u64) << 32 | (bottom as u64)
}
fn calibrate_tsc() {
    let mut sum = 0u64;
    let samples = 5u64;
    for _ in 0..samples {
        let start = tsc();
        wait(100);
        let end = tsc();
        sum += (end - start) * 10;
    }
    let hz = sum / samples;
    TSC_HZ.store(hz, core::sync::atomic::Ordering::Relaxed);
    serial_println!("hz: {}", hz);
}
pub fn get_ms() -> u64 {
    let hz = TSC_HZ.load(core::sync::atomic::Ordering::Relaxed);
    tsc() * 1000 / hz
}
pub fn get_us() -> u64 {
    let hz = TSC_HZ.load(core::sync::atomic::Ordering::Relaxed);
    tsc() * 1_000_000 / hz
}
fn sleep_ms(ms: u64) {
    let target = get_ms() + ms;
    while get_ms() < target {
        unsafe {
            core::arch::asm!("sti", "hlt", "cli");
        }
    }
    enable_interrupts();
}
