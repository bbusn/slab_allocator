<br>

<div align="center">

# slab allocator

A minimal slab allocator in rust `no_std`

[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0) [![Rust](https://img.shields.io/badge/rust-%23232.svg?logo=rust&logoColor=white)](https://rust-lang.org/)

<br><br>

</div>

## 📖 About

A simple slab allocator that manages fixed-size objects in pages. It's designed for `no_std` environments where you need efficient memory allocation without the standard library.

The allocator:

- Organizes memory into 4KB pages
- Maintains a free list for quick allocation/deallocation
- Automatically handles alignment requirements
- Reuses freed memory efficiently

## 💻 Installation

**Linux - x86_64**

```bash
cargo run --target x86_64-unknown-linux-gnu
```

**Linux - aarch64**

```bash
cargo run --target aarch64-unknown-linux-gnu
```

## 🚀 Usage

```rust
use slab_allocator::SlabAllocator;
use core::ptr::NonNull;

let mut allocator = SlabAllocator::new(64); // Allocate objects of size 64 bytes

// Allocate an object
if let Some(ptr) = allocator.alloc() {
    // Use the memory...

    // Free it when done
    allocator.free(ptr);
}
```

## 🧪 Testing

Run the test suite:

```bash
cargo test
```

<br><br>
