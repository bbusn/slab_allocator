#![no_std]
#![no_main]

pub mod sys;

#[cfg(test)]
extern crate std;

#[cfg(not(test))]
use core::panic::PanicInfo;

use crate::sys::exit;
use core::ptr;
use core::ptr::NonNull;
use core::ptr::addr_of_mut;

const PAGE_SIZE: usize = 4096;

// How many pages we can allocate
const MAX_PAGES: usize = 16;

// Memory pool for pages
static mut PAGE_POOL: [u8; MAX_PAGES * PAGE_SIZE] = [0; MAX_PAGES * PAGE_SIZE];
static mut PAGE_POOL_USED: usize = 0;

// Slab allocator struct.
pub struct SlabAllocator {
    object_size: usize,
    objects_per_page: usize,
    free_list: *mut FreeObject,
    pages: *mut Page,
}

/// Free list node stored inside free objects
struct FreeObject {
    next: *mut FreeObject,
}

// Page header with a pointer to the next page
#[repr(C)]
struct PageHeader {
    next: *mut PageHeader,
}

// A page is a header followed by the actual data
type Page = PageHeader;

impl SlabAllocator {
    pub fn new(object_size: usize) -> Self {
        // Make sure objects are at least pointer-sized (needed for free list)
        let object_size = object_size.max(core::mem::size_of::<*mut FreeObject>());

        // Align to pointer size
        let object_size = (object_size + core::mem::align_of::<*mut FreeObject>() - 1)
            & !(core::mem::align_of::<*mut FreeObject>() - 1);

        // Count how many objects fit in one page
        let usable_space = PAGE_SIZE - core::mem::size_of::<PageHeader>();
        let objects_per_page = usable_space / object_size;

        Self {
            object_size,
            objects_per_page,
            free_list: core::ptr::null_mut(),
            pages: core::ptr::null_mut(),
        }
    }

    pub fn alloc(&mut self) -> Option<NonNull<u8>> {
        // SAFETY: free_list is non-null after allocate_page, and points to valid memory from our pool.
        unsafe {
            // If free list is empty, allocate a new page
            if self.free_list.is_null() {
                self.allocate_page()?;
            }

            // Pop from free list
            let obj = self.free_list;
            self.free_list = (*obj).next;

            Some(NonNull::new_unchecked(obj as *mut u8))
        }
    }

    /// Free an object, returning it to the free list.
    pub fn free(&mut self, ptr: NonNull<u8>) {
        // SAFETY: ptr is validated to be within our pool before dereferencing.
        unsafe {
            let pool_start = addr_of_mut!(PAGE_POOL) as *const u8 as usize;
            let pool_end = pool_start + MAX_PAGES * PAGE_SIZE;

            let ptr_addr = ptr.as_ptr() as usize;

            // If pointer is outside the pool, ignore
            if ptr_addr < pool_start || ptr_addr >= pool_end {
                return;
            }

            let free_obj = ptr.as_ptr() as *mut FreeObject;
            (*free_obj).next = self.free_list;
            self.free_list = free_obj;
        }
    }

    unsafe fn allocate_page(&mut self) -> Option<()> {
        // Check if we have space for another page
        // SAFETY: Reading mutable static is safe because we're the only allocator.
        unsafe {
            if PAGE_POOL_USED + PAGE_SIZE > MAX_PAGES * PAGE_SIZE {
                return None;
            }
        }

        // Get the next page from the pool
        let pool_start = addr_of_mut!(PAGE_POOL) as *mut u8;
        // SAFETY: PAGE_POOL_USED is within bounds, and add stays within PAGE_POOL array.
        let page_ptr = unsafe { pool_start.add(PAGE_POOL_USED) } as *mut Page;
        // SAFETY: Writing to mutable static is safe because we're the only allocator.
        unsafe {
            PAGE_POOL_USED += PAGE_SIZE;
        }

        // Write the page header
        // SAFETY: page_ptr points to valid memory within PAGE_POOL that we just allocated.
        unsafe {
            ptr::write(page_ptr, Page { next: self.pages });
        }

        // The data area starts after the header
        // SAFETY: Adding header size stays within the page bounds.
        let data_start = unsafe { (page_ptr as *mut u8).add(core::mem::size_of::<PageHeader>()) };
        for i in 0..self.objects_per_page {
            // SAFETY: i * object_size is bounded by objects_per_page calculation.
            let obj_ptr = unsafe { data_start.add(i * self.object_size) } as *mut FreeObject;
            // SAFETY: obj_ptr points to valid memory within the page we just allocated.
            unsafe {
                (*obj_ptr).next = self.free_list;
            }
            self.free_list = obj_ptr;
        }

        self.pages = page_ptr;
        Some(())
    }
}

// SAFETY: This function is required by the C runtime ABI.
// It is not meant to be called directly; it exists only so the linker can resolve the symbol.
#[cfg(not(test))]
#[unsafe(no_mangle)]
pub extern "C" fn __libc_start_main() {
    main();
}

// SAFETY: This function is required by the C runtime ABI.
// It is not meant to be called directly; it exists only so the linker can resolve the symbol.
#[cfg(not(test))]
#[unsafe(no_mangle)]
pub extern "C" fn abort() {
    exit(1);
}

// SAFETY: This is the program entry point in a no_std environment.
// It is marked `no_mangle` so the linker can find it.
#[unsafe(no_mangle)]
pub extern "C" fn main() {
    let mut slab = SlabAllocator::new(64);

    let obj1 = slab.alloc();
    let obj2 = slab.alloc();
    let obj3 = slab.alloc();

    if let Some(ptr) = obj1 {
        slab.free(ptr);
    }
    if let Some(ptr) = obj2 {
        slab.free(ptr);
    }
    if let Some(ptr) = obj3 {
        slab.free(ptr);
    }

    let _obj4 = slab.alloc();
    let _obj5 = slab.alloc();

    exit(0);
}

// SAFETY: This is the panic handler required by Rust in a no_std environment.
// It is called automatically on panic.
#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    exit(1);
    loop {}
}

// SAFETY: This symbol is required by the Rust compiler for exception handling in a no_std environment.
// It can be left empty.
#[cfg(not(test))]
#[unsafe(no_mangle)]
pub extern "C" fn rust_eh_personality() {}

#[cfg(test)]
mod tests {
    use super::*;
    use core::ptr::NonNull;
    use std::vec::Vec;

    fn reset_state() {
        SlabAllocator::reset_pool();
    }

    #[test]
    fn test_free_outside_pool() {
        reset_state();
        let mut slab = SlabAllocator::new(64);

        // Create a fake pointer outside the pool
        let fake_ptr = NonNull::new(0xdeadbeef as *mut u8).unwrap();

        // Should not crash, just ignore
        slab.free(fake_ptr);

        // Allocator should still work
        let ptr = slab.alloc();
        assert!(ptr.is_some());
    }

    #[test]
    fn test_alloc_after_free() {
        reset_state();
        let mut slab = SlabAllocator::new(64);

        let mut ptrs = Vec::new();

        // Allocate several objects
        for _ in 0..10 {
            ptrs.push(slab.alloc().unwrap());
        }

        // Free all of them
        for ptr in ptrs {
            slab.free(ptr);
        }

        // Allocate again - should reuse freed memory
        let new_ptrs: Vec<_> = (0..10).map(|_| slab.alloc().unwrap()).collect();
        assert_eq!(new_ptrs.len(), 10);
    }

    #[test]
    fn test_object_size_alignment() {
        reset_state();
        // Test that object size is properly aligned
        let slab = SlabAllocator::new(13); // Not aligned to pointer size

        // object_size should be rounded up to at least pointer size
        assert!(slab.object_size >= core::mem::size_of::<*mut FreeObject>());
        assert_eq!(
            slab.object_size % core::mem::align_of::<*mut FreeObject>(),
            0
        );
    }

    #[test]
    fn test_multiple_pages() {
        reset_state();
        let mut slab = SlabAllocator::new(64);

        // Allocate enough objects to require multiple pages
        let objects_per_page = slab.objects_per_page;
        let mut ptrs = Vec::new();

        // Allocate objects from first page
        for _ in 0..objects_per_page {
            ptrs.push(slab.alloc().unwrap());
        }

        // This should trigger a new page allocation
        let ptr = slab.alloc();
        assert!(ptr.is_some(), "Should be able to allocate from new page");

        // Verify it's a different address
        let new_addr = ptr.unwrap().as_ptr() as usize;
        let first_addr = ptrs[0].as_ptr() as usize;
        assert_ne!(new_addr, first_addr);
    }
}
