#![no_std]
#![feature(sync_unsafe_cell)]
pub mod arch;
pub mod gdt;
pub mod idt;
pub mod memory;
pub mod serial;
