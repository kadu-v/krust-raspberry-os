#![no_std]
#![no_main]
#![feature(panic_info_message)]
#![feature(trait_alias)]
#![feature(const_fn_fn_ptr_basics)]
#![feature(format_args_nl)]

use core::panic;

use crate::console::interface::{Read, Statistics, Write};

mod bsp;
mod console;
mod cpu;
mod driver;
mod panic_wait;
mod print;
mod synchronization;
mod time;

//-------------------------------------------------------------------------------------------------
// Kernel code
//-------------------------------------------------------------------------------------------------
unsafe fn kernel_init() -> ! {
    use driver::interface::DriverManager;

    // Initialize all device
    for i in bsp::driver::driver_manager()
        .all_device_drivers()
        .into_iter()
    {
        if let Err(x) = i.init() {
            panic!("Error loading driver: {}: {}", i.compatible(), x)
        }
    }
    bsp::driver::driver_manager().post_device_driver_init();
    // println! is usable from here on

    // Trasmit from unsafe to safe
    kernel_main();
}

fn kernel_main() -> ! {
    use core::time::Duration;
    use driver::interface::DriverManager;
    use time::interface::TimeManager;

    info!(
        "{} version {}",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION")
    );
    info!("Booting on: {}", bsp::board_name());

    info!(
        "Architectural timer resolution: {} ns",
        time::time_manager().resolution().as_nanos()
    );

    info!("Drivers loaded:");
    for (i, driver) in bsp::driver::driver_manager()
        .all_device_drivers()
        .iter()
        .enumerate()
    {
        info!("      {}. {}", i + 1, driver.compatible());
    }

    // Test a failing timer case.
    time::time_manager().spin_for(Duration::from_nanos(1));

    loop {
        info!("Spinning for 1 second");
        time::time_manager().spin_for(Duration::from_secs(1));
    }
}
