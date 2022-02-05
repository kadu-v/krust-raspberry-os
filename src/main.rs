#![no_std]
#![no_main]
#![feature(panic_info_message)]
#![feature(trait_alias)]
#![feature(const_fn_fn_ptr_basics)]
#![feature(format_args_nl)]
#![feature(core_intrinsics)]

use core::panic;

use crate::console::interface::{Read, Statistics, Write};

mod bsp;
mod console;
mod cpu;
mod driver;
mod exception;
mod memory;
mod panic_wait;
mod print;
mod synchronization;
mod time;

//-------------------------------------------------------------------------------------------------
// Kernel code
//-------------------------------------------------------------------------------------------------
unsafe fn kernel_init() -> ! {
    use driver::interface::DriverManager;
    use memory::mmu::interface::MMU;

    if let Err(string) = memory::mmu::mmu().enable_mmu_and_caching() {
        panic!("MMU: {}", string);
    }

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

    info!("MMU online. Special regions:");
    bsp::memory::mmu::virt_mem_layout().print_layout();

    let (_, privilege_level) = exception::current_privilege_level();
    info!("Current privilege level: {}", privilege_level);

    info!("Exception handling state: ");
    exception::asynchronous::print_state();

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

    let remapped_uart = unsafe { bsp::device_driver::PL011Uart::new(0x1FFF_1000) };
    writeln!(
        remapped_uart,
        "[     !!!    ] Writing through the remapped UART at 0x1FFF_1000"
    )
    .unwrap();

    loop {
        info!("Spinning for 1 second");
        time::time_manager().spin_for(Duration::from_secs(1));
    }
}
