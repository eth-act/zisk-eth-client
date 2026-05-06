use core::alloc::{GlobalAlloc, Layout};

// critical-section implementation for single-threaded bare-metal: no-op.
// Required by once_cell (pulled in via reth-evm-ethereum) on targets without OS support.
#[unsafe(no_mangle)]
unsafe extern "C" fn _critical_section_1_0_acquire() -> u8 {
    0 // no saved state; interrupts not in use
}

#[unsafe(no_mangle)]
unsafe extern "C" fn _critical_section_1_0_release(_token: u8) {}

unsafe extern "C" {
    static _kernel_heap_bottom: u8;
    static _kernel_heap_top: u8;
}

static mut HEAP_POS: usize = 0;
static mut HEAP_TOP: usize = 0;

/// Initialize the heap from linker-script symbols. Must be called before any allocation.
pub unsafe fn init_heap() {
    unsafe {
        HEAP_POS = &raw const _kernel_heap_bottom as *const u8 as usize;
        HEAP_TOP = &raw const _kernel_heap_top as *const u8 as usize;
    }
}

#[inline(always)]
unsafe fn bump_alloc(bytes: usize, align: usize) -> *mut u8 {
    let mut pos = unsafe { HEAP_POS };
    let offset = pos & (align - 1);
    if offset != 0 {
        pos += align - offset;
    }
    let ptr = pos as *mut u8;
    pos += bytes;
    if unsafe { HEAP_TOP } < pos {
        panic!("OOM: heap exhausted");
    }
    unsafe { HEAP_POS = pos };
    ptr
}

pub struct BumpPointerAlloc;

unsafe impl GlobalAlloc for BumpPointerAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        unsafe { bump_alloc(layout.size(), layout.align()) }
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {}

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        unsafe { bump_alloc(layout.size(), layout.align()) }
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let new_ptr = unsafe { bump_alloc(new_size, layout.align()) };
        unsafe { core::ptr::copy_nonoverlapping(ptr, new_ptr, layout.size().min(new_size)) };
        new_ptr
    }
}

#[global_allocator]
static ALLOC: BumpPointerAlloc = BumpPointerAlloc;
