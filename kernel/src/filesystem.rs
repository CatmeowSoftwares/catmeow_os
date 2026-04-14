use core::slice;

use limine::file::File;

use crate::{arch::arch::loop_halt, serial_println, terminal_println};

pub fn init_filesystem(file: &File) {
    let data = unsafe { slice::from_raw_parts_mut(file.addr(), file.size() as usize) };
    let result = tar_no_std::TarArchiveRef::new(data);
    match result {
        Ok(val) => {
            return;
            for i in val.entries() {
                serial_println!("{}", i.filename().as_str().unwrap());
                if i.filename().as_str().unwrap() == "root/meow.bin" {}
            }
        }
        Err(err) => {
            serial_println!("{}", err)
        }
    }
    //loop_halt();
}
