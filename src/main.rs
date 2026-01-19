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
        let usable_space = PAGE_SIZE - core::mem::size_of::<Page>();
        let objects_per_page = usable_space / object_size;
        
        Self {
            object_size,
            objects_per_page,
            free_list: core::ptr::null_mut(),
            pages: core::ptr::null_mut(),
        }
    }

    pub fn alloc(&mut self) -> Option<NonNull<u8>> {
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
        if PAGE_POOL_USED + PAGE_SIZE > MAX_PAGES * PAGE_SIZE {
            return None;
        }

        // Get the next page from the pool
        let pool_start = addr_of_mut!(PAGE_POOL) as *mut u8;
        let page_ptr = pool_start.add(PAGE_POOL_USED) as *mut Page;
        PAGE_POOL_USED += PAGE_SIZE;

        // Write the page header
        ptr::write(page_ptr, Page {
            next: self.pages,
        });

        // The data area starts after the header
        let data_start = (page_ptr as *mut u8).add(core::mem::size_of::<Page>());
        for i in 0..self.objects_per_page {
            let obj_ptr = data_start.add(i * self.object_size) as *mut FreeObject;
            (*obj_ptr).next = self.free_list;
            self.free_list = obj_ptr;
        }

        self.pages = page_ptr;
        Some(())
    }
}

// SAFETY: This function is required by the C runtime ABI.
// It is not meant to be called directly; it exists only so the linker can resolve the symbol.
#[cfg(not(test))]
#[no_mangle]
pub extern "C" fn __libc_start_main() {
    main();
}

// SAFETY: This function is required by the C runtime ABI.
// It is not meant to be called directly; it exists only so the linker can resolve the symbol.
#[cfg(not(test))]
#[no_mangle]
pub extern "C" fn abort() {
    exit(1);
}

// SAFETY: This is the program entry point in a no_std environment.
// It is marked `no_mangle` so the linker can find it.
#[no_mangle]
pub extern "C" fn main() {
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
#[no_mangle]
pub extern "C" fn rust_eh_personality() {}
