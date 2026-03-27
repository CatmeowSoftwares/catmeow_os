//bitmap allocator
// n/8 bytes

use core::{cell::SyncUnsafeCell, ptr::null_mut};

use crate::{linked_list::LinkedList, serial_println};
use limine::memory_map::{Entry, EntryType};
#[derive(Debug, Clone, Copy)]
struct PageFrame {
    start: u64,
    size: u64,
    used: bool,
}

struct PageTable1 {}
struct PageTable2 {
    page_1: PageTable1,
}
struct PageTable3 {
    page_2: PageTable2,
}
struct PageTable4 {
    page_3: PageTable3,
}
impl PageFrame {
    const fn new(start: u64, size: u64) -> Self {
        Self {
            start,
            size,
            used: false,
        }
    }
    fn free(&mut self) {
        self.used = false;
    }
    fn is_empty(&self) -> bool {
        self.size == 0
    }
}
static FRAME_MAP: SyncUnsafeCell<[PageFrame; 1024]> =
    SyncUnsafeCell::new([PageFrame::new(0, 0); 1024]);
#[derive(Debug, Clone, Copy)]
struct Region {
    previous_addr: u64,
    next_addr: u64,
    size: u64,
}

impl Region {
    const fn new(previous_addr: u64, size: u64) -> Self {
        Self {
            previous_addr,
            size,
            next_addr: 0,
        }
    }
    fn is_empty(&self) -> bool {
        self.size == 0
    }
}
struct ListNode {
    next: Option<&'static ListNode>,
    previous_addr: u64,
    next_addr: u64,
}
impl ListNode {
    const fn new(prev: u64, next: u64) -> Self {
        Self {
            next: None,
            previous_addr: prev,
            next_addr: next,
        }
    }
    fn as_u64(&self) -> u64 {
        self.as_ptr() as u64
    }
    fn as_ptr(&self) -> *const ListNode {
        self as *const ListNode
    }
    fn as_ptr_mut(&mut self) -> *mut ListNode {
        self as *mut ListNode
    }
}
struct FreelistAllocator {
    head: ListNode,
}
const PAGE_SIZE: u16 = 4096;
impl FreelistAllocator {
    const fn new() -> Self {
        Self {
            head: ListNode::new(0, 0),
        }
    }
    fn alloc(&mut self) -> *mut u8 {
        let mut node = ListNode::new(0, 0);
        let node_ptr = node.as_ptr_mut();
        unsafe {
            self.head.next = Some(&mut *node_ptr);
        }

        null_mut()
    }
    fn free(&mut self) {}
}
static FREELIST_ALLOCATOR: SyncUnsafeCell<FreelistAllocator> =
    SyncUnsafeCell::new(FreelistAllocator::new());
static FREE_MEMORY_REGIONS: SyncUnsafeCell<[Region; 256]> =
    SyncUnsafeCell::new([Region::new(0, 0); 256]);
pub fn init_memory(entries: &[&Entry]) {
    let mut lili = LinkedList::new(0u64);
    lili.add(67);
    lili.add(41);
    lili.add(420);
    let mut current = &lili.head;
    loop {
        if current.next.is_null() {
            serial_println!("current is null");
            //break;
        }
        let node = unsafe { &*current.next };
        serial_println!("test while let");
        serial_println!("val is: {}", node.item);
        current = &node;
    }
    let frame_map = unsafe { &mut *FRAME_MAP.get() };
    let free_memory_region = unsafe { &mut *FREE_MEMORY_REGIONS.get() };
    for (i, entry) in entries.iter().enumerate() {
        if entry.entry_type == EntryType::USABLE {
            serial_println!("USABLE: base: {}, length: {}", entry.base, entry.length);
            frame_map[i] = PageFrame::new(entry.base, entry.length);
            let region = Region::new(entry.base, entry.length);
        } else if entry.entry_type == EntryType::RESERVED {
            serial_println!("RESERVED base: {}, length: {}", entry.base, entry.length);
        }
    }
    //test()
}

fn test() {
    let a = 0xdeadbeefu64 as *mut u8;
    unsafe {
        *a = 67;
        serial_println!("{}", *a);
    }
}

enum PageState {
    Uninit,
    Used,
    Free,
}
struct Page {
    state: PageState,
}
struct PageTable<'a> {
    pages: &'a [Page],
}
