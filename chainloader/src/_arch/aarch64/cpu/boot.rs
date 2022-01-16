// Assembly counterpart to this file.
core::arch::global_asm!(include_str!("boot.s"));

//-------------------------------------------------------------------------------------------------
// Public Code
//-------------------------------------------------------------------------------------------------
/// The Rust entry `kernel` binary
///
/// The function is called from thw assembley `_start` function

#[no_mangle]
pub unsafe fn _start_rust() -> ! {
    crate::kernel_init()
}
