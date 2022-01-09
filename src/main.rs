#![no_std]
#![no_main]
#![feature(panic_info_message)]

use core::panic;

mod bsp;
mod console;
mod cpu;
mod panic_wait;
mod print;

//-------------------------------------------------------------------------------------------------
// Kernel code
//-------------------------------------------------------------------------------------------------
fn kernel_init() -> ! {
    println!("[0] Hello from Rust!");

    panic!("Stopping here.");
}
