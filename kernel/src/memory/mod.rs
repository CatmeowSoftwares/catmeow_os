use core::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

use crate::memory::{
    pmm::{get_hhdm_offset, page_align},
    vmm::map_page,
};

pub mod heap;
pub mod pmm;
pub mod vmm;

pub const PAGE_SIZE: u64 = 4096;

// TODO: free and idk stuff
pub struct PhysicalPointer<T: Sized> {
    paddr: usize,
    _phantom: PhantomData<*mut T>,
}

impl<T: Copy> PhysicalPointer<T> {
    pub fn new(addr: usize) -> Self {
        let phys_ptr = Self {
            paddr: addr,
            _phantom: PhantomData,
        };
        let aligned_phys = page_align(phys_ptr.paddr as u64);
        let aligned_virt = aligned_phys + get_hhdm_offset();
        map_page(aligned_phys as *mut u8, aligned_virt as *mut u8, 3);
        phys_ptr
    }
    pub unsafe fn read(&self) -> T {
        unsafe { *((self.paddr + get_hhdm_offset() as usize) as *const T) }
    }
    pub unsafe fn write(&self, value: T) {
        unsafe { *((self.paddr + get_hhdm_offset() as usize) as *mut T) = value };
    }
    pub unsafe fn ptr(&self) -> *const T {
        (self.paddr + get_hhdm_offset() as usize) as *const T
    }
    pub fn address(&self) -> usize {
        self.paddr
    }
}

impl<T> Deref for PhysicalPointer<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { &*((self.paddr + get_hhdm_offset() as usize) as *const T) }
    }
}

impl<T> DerefMut for PhysicalPointer<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *((self.paddr + get_hhdm_offset() as usize) as *mut T) }
    }
}

impl<T> Drop for PhysicalPointer<T> {
    fn drop(&mut self) {
        //unmap_page(page_align(self.paddr as u64 + get_hhdm_offset()) as *mut u8);
    }
}
