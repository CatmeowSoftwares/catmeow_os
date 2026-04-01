use core::{arch::asm, fmt::Write};
const PORT: u16 = 0x3F8;

struct Serial;

impl Write for Serial {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let t = s.as_bytes();
        for &c in t {
            write_serial(c);
        }
        Ok(())
    }
}
pub fn init_serial() -> i32 {
    unsafe {
        outb(PORT + 1, 0x00); // Disable all interrupts
        outb(PORT + 3, 0x80); // Enable DLAB (set baud rate divisor)
        outb(PORT + 0, 0x03); // Set divisor to 3 (lo byte) 38400 baud
        outb(PORT + 1, 0x00); //                  (hi byte)
        outb(PORT + 3, 0x03); // 8 bits, no parity, one stop bit
        outb(PORT + 2, 0xC7); // Enable FIFO, clear them, with 14-byte threshold
        outb(PORT + 4, 0x0B); // IRQs enabled, RTS/DSR set
        outb(PORT + 4, 0x1E); // Set in loopback mode, test the serial chip
        outb(PORT + 0, 0xAE); // Test serial chip (send byte 0xAE and check if serial returns same byte)

        if inb(PORT + 0) != 0xAE {
            return 1;
        }

        // If serial is not faulty set it in normal operation mode
        // (not-loopback with IRQs enabled and OUT#1 and OUT#2 bits enabled)
        outb(PORT + 4, 0x0F);
    }
    0
}
pub unsafe fn inb(port: u16) -> u8 {
    let value: u8;
    unsafe {
        asm!("in al, dx", out("al") value, in("dx") port, options(nomem, nostack, preserves_flags));
    }
    value
}

pub unsafe fn outb(port: u16, value: u8) {
    unsafe {
        asm!("out dx, al", in("dx") port, in("al") value, options(nomem, nostack, preserves_flags));
    }
}

fn is_transmit_empty() -> u8 {
    unsafe { inb(PORT + 5) & 0x20 }
}

fn write_serial(a: u8) {
    while is_transmit_empty() == 0 {}

    unsafe {
        outb(PORT, a);
    }
}
#[doc(hidden)]
pub fn _print(args: ::core::fmt::Arguments) {
    Serial.write_fmt(args).expect("failed to write in serial");
}

#[macro_export]
macro_rules! serial_print {
    () => {
        $crate::print!("\n")
    };
    ($($arg:tt)*) => {{
        $crate::serial::_print(format_args!($($arg)*));
    }};
}

#[macro_export]
macro_rules! serial_println {
    () => ($crate::serial_print!("\n"));
    ($fmt:expr) => ($crate::serial_print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => ($crate::serial_print!(
        concat!($fmt, "\n"), $($arg)*));
}
