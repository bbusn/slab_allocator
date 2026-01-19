#![no_std]
#![no_main]

pub mod sys;

#[cfg(test)]
extern crate std;

#[cfg(not(test))]
use core::panic::PanicInfo;

use crate::sys::exit;

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
