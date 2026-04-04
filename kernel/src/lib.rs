#![no_std]
#![feature(sync_unsafe_cell)]
extern crate alloc;
pub mod acpi;
pub mod arch;
pub mod gdt;
pub mod idt;
pub mod memory;
pub mod serial;
