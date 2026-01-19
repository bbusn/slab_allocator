#![no_std]
#![no_main]

pub mod sys;

#[cfg(test)]
extern crate std;

#[cfg(not(test))]
use core::panic::PanicInfo;

use crate::sys::exit;
use core::ptr;

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

        Self {
            object_size,
            objects_per_page: 0,
            free_list: core::ptr::null_mut(),
            pages: core::ptr::null_mut(),
        }
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
