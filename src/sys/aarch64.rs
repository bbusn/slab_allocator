pub mod syscalls {
    pub const EXIT: usize = 93;
}

/* __________ Syscalls __________ */
#[inline(always)]
pub fn syscall_1(n: usize, a0: usize) -> isize {
    let ret: isize;

    // SAFETY: We are issuing a raw syscall via inline assembly.
    // The arguments are placed in the correct registers as required by the
    // aarch64 convention, and we only use the return value from x0.
    unsafe {
        core::arch::asm!(
            "svc 0",
            in("x8") n,
            in("x0") a0,
            lateout("x0") ret,
            options(nostack)
        );
    }
    ret
}

/* __________ Helpers __________ */
pub fn exit(code: usize) {
    syscall_1(syscalls::EXIT, code);
}