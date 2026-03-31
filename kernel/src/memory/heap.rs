use core::{alloc::GlobalAlloc, cell::SyncUnsafeCell};

use crate::{memory::pmm, serial_println};

struct Allocator {}
impl Allocator {
    const fn new() -> Self {
        Self {}
    }

    fn pop(&mut self) -> *mut u8 {
        pmm::allocate()
    }
    fn push(&mut self, ptr: *mut u8) {
        pmm::free(ptr);
    }
}

unsafe impl GlobalAlloc for MySyncUnsafeCellAllocator {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        let allocator = self.get_mut();
        allocator.pop()
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: core::alloc::Layout) {
        let allocator = self.get_mut();
        allocator.push(ptr);
    }
}

#[global_allocator]
static ALLOCATOR: MySyncUnsafeCellAllocator = MySyncUnsafeCellAllocator::new();

struct MySyncUnsafeCellAllocator {
    inner: SyncUnsafeCell<Allocator>,
}

impl MySyncUnsafeCellAllocator {
    const fn new() -> Self {
        Self {
            inner: SyncUnsafeCell::new(Allocator::new()),
        }
    }

    fn get_mut(&self) -> &mut Allocator {
        unsafe { &mut *self.inner.get() }
    }
}
