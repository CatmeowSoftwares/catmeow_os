use core::{
    alloc::{GlobalAlloc, Layout},
    ptr::null_mut,
};

use spin::{Mutex, MutexGuard};

use crate::{
    memory::{pmm, vmm},
    terminal_println,
};
const HEAP_START: usize = 0xFFFF_8000_0000_0000;
const HEAP_SIZE: usize = 4 * 1024 * 1024;
struct Allocator {
    head: Node,
}
struct Node {
    size: usize,
    next: Option<&'static mut Node>,
}
impl Node {
    const fn new(size: usize) -> Self {
        Self { size, next: None }
    }
    fn get_start(&self) -> usize {
        self as *const Self as usize
    }
    fn get_end(&self) -> usize {
        self.get_start() + self.size
    }
}
impl Allocator {
    const fn new() -> Self {
        Self { head: Node::new(0) }
    }
    fn size_align(layout: Layout) -> (usize, usize) {
        let layout = layout
            .align_to(align_of::<Node>())
            .expect("failed to align")
            .pad_to_align();
        let size = layout.size().max(size_of::<Node>());
        (size, layout.align())
    }

    fn add(&mut self, start: usize, size: usize) {
        let mut node = Node::new(size);
        node.next = self.head.next.take();
        let ptr = start as *mut Node;
        unsafe {
            ptr.write(node);
            self.head.next = Some(&mut *ptr);
        }
    }
    fn remove(&mut self, ptr: *mut u8) {
        let start = ptr as u64;
    }
    fn find(&mut self, size: usize, align: usize) -> Option<(&mut Node, usize)> {
        let mut current = &mut self.head;
        while let Some(ref mut region) = current.next {
            if let Some(start) = Self::allocate_from_region(&region, size, align) {
                let next = region.next.take();
                let ret = Some((current.next.take().unwrap(), start));
                current.next = next;
                return ret;
            } else {
                current = current.next.as_mut().unwrap();
            }
        }
        None
    }
    fn allocate_from_region(region: &Node, size: usize, align: usize) -> Option<usize> {
        let val = Self::align_up(region.get_start(), align);
        let end = val.checked_add(size)?;
        if end > region.get_end() {
            return None;
        }
        let excess = region.get_end() - end;
        if excess > 0 && excess < size_of::<Node>() {
            return None;
        }
        Some(val)
    }
    fn align_up(addr: usize, align: usize) -> usize {
        (addr + align - 1) & !(align - 1)
    }
}

unsafe impl GlobalAlloc for MyAllocator {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        let (size, align) = Allocator::size_align(layout);
        let mut allocator = self.lock();
        if let Some((region, start)) = allocator.find(size, align) {
            let alloc_end = start.checked_add(size).expect("overflow");
            let excess = region.get_end() - alloc_end;
            if excess > 0 {
                allocator.add(alloc_end, excess);
            }
            start as *mut u8
        } else {
            null_mut()
        }
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: core::alloc::Layout) {
        let mut allocator = self.lock();
        let (size, _) = Allocator::size_align(layout);
        allocator.add(ptr as usize, size);
    }
}

#[global_allocator]
static ALLOCATOR: MyAllocator = MyAllocator::new();

struct MyAllocator {
    inner: Mutex<Allocator>,
}

impl MyAllocator {
    const fn new() -> Self {
        Self {
            inner: Mutex::new(Allocator::new()),
        }
    }

    fn lock(&self) -> MutexGuard<'_, Allocator> {
        self.inner.lock()
    }
}

pub fn init_heap() {
    terminal_println!("heap init start");
    vmm::allocate(HEAP_START as u64, HEAP_SIZE, 3);
    terminal_println!("heap allocate done");
    let mut allocator = ALLOCATOR.lock();
    allocator.add(HEAP_START, HEAP_SIZE);
    terminal_println!("heap init good");
}
