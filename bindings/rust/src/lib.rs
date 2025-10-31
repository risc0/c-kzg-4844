#![cfg_attr(not(feature = "std"), no_std)]

#[macro_use]
extern crate alloc;

// This `extern crate` invocation tells `rustc` that we actually need the symbols from `blst`.
// Without it, the compiler won't link to `blst` when compiling this crate.
// See: https://kornel.ski/rust-sys-crate#linking
extern crate blst;

mod bindings;

#[cfg(feature = "ethereum_kzg_settings")]
mod ethereum_kzg_settings;

// Expose relevant types with idiomatic names.
pub use bindings::{
    KZGCommitment as KzgCommitment, KZGProof as KzgProof, KZGSettings as KzgSettings,
    C_KZG_RET as CkzgError,
};

// Expose the default settings.
#[cfg(feature = "ethereum_kzg_settings")]
pub use ethereum_kzg_settings::ethereum_kzg_settings;

// Expose the constants.
pub use bindings::{
    BYTES_PER_BLOB, BYTES_PER_CELL, BYTES_PER_COMMITMENT, BYTES_PER_FIELD_ELEMENT, BYTES_PER_PROOF,
    CELLS_PER_EXT_BLOB, FIELD_ELEMENTS_PER_BLOB, FIELD_ELEMENTS_PER_CELL,
};
// Expose the remaining relevant types.
pub use bindings::{Blob, Bytes32, Bytes48, Cell, Error};

#[cfg(all(feature = "risc0-ffi", target_os = "zkvm"))]
mod risc0_ffi {
    extern crate alloc;

    const LEN_SIZE: usize = core::mem::size_of::<usize>();

    unsafe fn store_len(block_ptr: *mut u8, size: usize) -> *mut u8 {
        let len_ptr = block_ptr as *mut usize;
        *len_ptr = size;

        block_ptr.offset(LEN_SIZE as isize)
    }

    unsafe fn get_len(block_ptr: *mut u8) -> (usize, *mut u8) {
        let len_ptr = block_ptr.offset(-(LEN_SIZE as isize)) as *mut usize;
        (*len_ptr, len_ptr as *mut u8)
    }

    #[no_mangle]
    unsafe extern "C" fn malloc(size: usize) -> *mut core::ffi::c_void {
        let layout = core::alloc::Layout::from_size_align(size + LEN_SIZE, 4)
            .expect("unable to construct memory layout");
        let block_ptr = alloc::alloc::alloc(layout);

        if block_ptr.is_null() {
            alloc::alloc::handle_alloc_error(layout);
        }

        let data_ptr = store_len(block_ptr, size);

        data_ptr as *mut core::ffi::c_void
    }

    #[no_mangle]
    unsafe extern "C" fn calloc(nobj: usize, size: usize) -> *mut core::ffi::c_void {
        let size = nobj * size;
        let layout = core::alloc::Layout::from_size_align(size + LEN_SIZE, 4)
            .expect("unable to construct memory layout");
        let block_ptr = alloc::alloc::alloc_zeroed(layout);

        if block_ptr.is_null() {
            alloc::alloc::handle_alloc_error(layout);
        }

        let data_ptr = store_len(block_ptr, size);

        data_ptr as *mut core::ffi::c_void
    }

    #[no_mangle]
    unsafe extern "C" fn free(block_ptr: *const core::ffi::c_void) {
        let (size, free_ptr) = get_len(block_ptr as *mut u8);

        let layout = core::alloc::Layout::from_size_align(size, 4)
            .expect("unable to construct memory layout");

        alloc::alloc::dealloc(free_ptr, layout);
    }

    #[no_mangle]
    pub extern "C" fn __assert_func(
        _file: *const i8,
        _line: i32,
        _func: *const i8,
        _expr: *const i8,
    ) {
        panic!("c_kzg assertion failure.");
    }
}
