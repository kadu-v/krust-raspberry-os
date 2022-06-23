#![feature(format_args_nl)]
#![no_main]
#![no_std]

use exception::asynchronous::interface::IRQManager;
use libkernel::{
    bsp::{self, frame_buffer::screen},
    driver, exception, info, memory,
    screen::interface::Write,
    state, time, warn,
};
use noto_sans_mono_bitmap::{
    get_bitmap, get_bitmap_width, BitmapHeight, FontWeight,
};

//-------------------------------------------------------------------------------------------------
// Kernel code
//-------------------------------------------------------------------------------------------------
#[no_mangle]
unsafe fn kernel_init() -> ! {
    use driver::interface::DriverManager;
    use memory::mmu::interface::MMU;

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

    // print font to screen
    let width = get_bitmap_width(FontWeight::Regular, BitmapHeight::Size64);
    info!(
        "Each char of the mono-spaced font will be {}px in width if the font \
         weight={:?} and the bitmap height={}",
        width,
        FontWeight::Regular,
        BitmapHeight::Size64.val()
    );
    let bitmap_char =
        get_bitmap('A', FontWeight::Regular, BitmapHeight::Size64)
            .expect("unsupported char");
    info!("{:?}", bitmap_char);
    for (row_i, row) in bitmap_char.bitmap().iter().enumerate() {
        for (col_i, intensity) in row.iter().enumerate() {
            let (r, g, b) =
                (*intensity as u32, *intensity as u32, *intensity as u32);
            let (r, g, b) = (255 - r, 255 - g, 255 - b);
            // let rgb_32 = /*0 << 24 | */r << 11 | g << 6 | b;
            let rgb_32 = /*0 << 24 | */r << 16 | g << 8 | b;
            screen().draw(col_i, row_i, rgb_32);
            info!("r: {}, g: {}, b: {}", r, g, b);
        }
    }

    // for i in 0..100 {
    //     for j in 0..100 {
    //         screen().draw(i, j, 0b11111_000000_00000);
    //     }
    // }
    loop {
        info!("Spinning for 1 second");
        time::time_manager().spin_for(Duration::from_secs(1));
    }
}
