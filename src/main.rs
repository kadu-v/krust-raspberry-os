#![feature(format_args_nl)]
#![no_main]
#![no_std]

use libkernel::{
    bsp::{self, frame_buffer::screen},
    driver, exception, info, memory, print, println,
    screen::interface::Write,
    time,
};

//-------------------------------------------------------------------------------------------------
// Kernel code
//-------------------------------------------------------------------------------------------------
#[no_mangle]
unsafe fn kernel_init() -> ! {
    use driver::interface::{DeviceDriver, DriverManager};
    // use memory::mmu::interface::MMU;

    // if let Err(string) = memory::mmu::mmu().enable_mmu_and_caching() {
    //     panic!("MMU: {}", string);
    // }

    exception::handling_init();

    // Initialize all device
    for (_, i) in bsp::driver::driver_manager()
        .all_device_drivers()
        .into_iter()
        .enumerate()
    {
        if let Err(e) = i.init() {
            panic!("Error loading driver: {}: {}", i.compatible(), e)
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

    for i in 0..100 {
        for j in 0..100 {
            screen().draw(i, j, 0b11111_000000_00000);
        }
    }
    loop {
        info!("Spinning for 1 second");
        time::time_manager().spin_for(Duration::from_secs(1));
    }
}
