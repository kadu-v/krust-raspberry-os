#![no_std]
#![no_main]
#![feature(panic_info_message)]
#![feature(trait_alias)]

use core::panic;

use crate::console::interface::Statistics;

mod bsp;
mod console;
mod cpu;
mod panic_wait;
mod print;
mod synchronization;

//-------------------------------------------------------------------------------------------------
// Kernel code
//-------------------------------------------------------------------------------------------------
fn kernel_init() -> ! {
    println!("[0] Hello from Rust!");

    println!(
        "[1] Chars written: {}",
        bsp::console::console().chars_written()
    );

    println!("[2] Stopping here.");

    cpu::wait_forever();
}
