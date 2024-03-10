#![feature(format_args_nl)]
#![no_main]
#![no_std]

use exception::asynchronous::interface::IRQManager;
use libkernel::{bsp, driver, exception, info, memory, state, time, warn};

//-------------------------------------------------------------------------------------------------
// Kernel code
//-------------------------------------------------------------------------------------------------
#[no_mangle]
unsafe fn kernel_init() -> ! {
    use driver::interface::DriverManager;
    use memory::mmu::interface::MMU;
    // use driver::{FRAMEBUFFER, MAILBOX};

    if let Err(string) = memory::mmu::mmu().enable_mmu_and_caching() {
        panic!("MMU: {}", string);
    }

    exception::handling_init();

    // Initialize all device
    for (_, i) in bsp::driver::driver_manager()
        .all_device_drivers()
        .into_iter()
        .enumerate()
    {
        if let Err(e) = i.init() {
            panic!("Error loading driver: {}: {}", i.compatible(), e)
        } else {
            info!("Load {}", i.compatible());
        }
    }
    bsp::driver::driver_manager().post_device_driver_init();
    // println! is usable from here on
    // Trasmit from unsafe to safe
    // Let device drivers register and enable their handlers with the interrupt controller.
    for i in bsp::driver::driver_manager().all_device_drivers() {
        if let Err(msg) = i.register_and_enable_irq_handler() {
            warn!("Error registering IRQ handler: {}", msg);
        }
    }

    // Unmask interrupts on the boot CPU core.
    exception::asynchronous::local_irq_unmask();

    // Announce conclusion of the kernel_init() phase.
    state::state_manager().transition_to_single_core_main();
    info!("Start Kernel");
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

    info!("Registered IRQ handlers:");
    bsp::exception::asynchronous::irq_manager().print_handler();

    // Test a failing timer case.
    time::time_manager().spin_for(Duration::from_nanos(1));

    // let mut b = true;
    // loop {
    //     info!("Spinning for 1 second");
    //     time::time_manager().spin_for(Duration::from_secs(1));
    //     b = !b;

    //     for b in ['A', 'B'] {
    //         for x in 0..(1023 / 8) {
    //             buffer_println!("hi");
    //         }
    //     }
    // }

    loop {
        info!("Spinning for 1 second");
        time::time_manager().spin_for(Duration::from_secs(1));
    }
}
