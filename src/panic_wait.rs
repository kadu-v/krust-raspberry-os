use crate::bsp;
use crate::cpu;

use core::fmt;
use core::panic::PanicInfo;

//--------------------------------------------------------------------------------------------------
// Private Code
//--------------------------------------------------------------------------------------------------

fn _panic_print(args: fmt::Arguments) {
    use fmt::Write;

    unsafe {
        bsp::console::panic_console_out().write_fmt(args).unwrap();
    };
}

#[macro_export]
macro_rules! panic_println {
    ($($arg:tt)*) => {{
            _panic_print(format_args_nl!($($arg)*))
        }};
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    if let Some(args) = info.message() {
        panic_println!("\nKernel panic: {}", args);
    } else {
        panic_println!("\nKernel panic!");
    }

    cpu::wait_forever();
}
