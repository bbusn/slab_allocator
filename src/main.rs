#![no_std]
#![no_main]

#[cfg(test)]
extern crate std;

#[cfg(not(test))]
use core::panic::PanicInfo;

// Entry point for aarch64
#[cfg(not(test))]
#[unsafe(no_mangle)]
fn __libc_start_main() {
    main();
}

// Required by the compiler
#[cfg(not(test))]
#[unsafe(no_mangle)]
fn abort() {}

#[unsafe(no_mangle)]
fn main() {}

// Panic handler required for no_std
#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {};
}

// Required by the compiler (not actually called)
#[cfg(not(test))]
#[unsafe(no_mangle)]
pub extern "C" fn rust_eh_personality() {}
