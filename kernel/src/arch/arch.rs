use core::arch::asm;
pub fn loop_halt() {
    loop {
        hlt();
    }
}
pub fn halt() {
    hlt()
}

pub fn hlt() {
    unsafe {
        #[cfg(target_arch = "x86_64")]
        asm!("hlt");
        #[cfg(any(target_arch = "aarch64", target_arch = "riscv64"))]
        asm!("wfi");
        #[cfg(target_arch = "loongarch64")]
        asm!("idle 0");
    }
}

pub fn init() {
    #[cfg(target_arch = "x86_64")]
    #[cfg(any(target_arch = "aarch64", target_arch = "riscv64"))]
    #[cfg(target_arch = "loongarch64")]
    1;
}
