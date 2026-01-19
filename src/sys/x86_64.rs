pub mod syscalls {
    pub const EXIT: usize = 60;
}

/* __________ Syscalls __________ */
#[inline(always)]
pub fn syscall_1(n: usize, a0: usize) -> isize {
    let ret: isize;
    // SAFETY: We are issuing a raw syscall via inline assembly.
    // The arguments are placed in the correct registers as required by the
    // x86_64 syscall convention, and we only use the return value from rax.
    unsafe {
        core::arch::asm!(
            "syscall",
            in("rax") n,
            in("rdi") a0,
            lateout("rax") ret,
            options(nostack)
        );
    }
    ret
}

/* __________ Helpers __________ */
pub fn exit(code: usize) {
    syscall_1(syscalls::EXIT, code);
}