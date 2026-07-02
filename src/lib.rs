#![no_std]
#![no_main]
#![feature(trait_alias)]
#![feature(format_args_nl)]
#![feature(linkage)]
mod panic_wait;
mod synchronization;

pub mod bsp;
pub mod console;
pub mod cpu;
pub mod driver;
pub mod exception;
pub mod memory;
pub mod print;
pub mod screen;
pub mod state;
pub mod time;

//--------------------------------------------------------------------------------------------------
// Public Code
//--------------------------------------------------------------------------------------------------

/// Version string.
pub fn version() -> &'static str {
    concat!(
        env!("CARGO_PKG_NAME"),
        " version ",
        env!("CARGO_PKG_VERSION")
    )
}

#[cfg(not(test))]
extern "Rust" {
    fn kernel_init() -> !;
}
