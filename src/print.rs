use crate::{bsp, screen};
use core::fmt;

//-------------------------------------------------------------------------------------------------
// Public Deginitions
//-------------------------------------------------------------------------------------------------

// #[doc(hidden)]
// pub fn _print(args: fmt::Arguments) {
//     use console::interface::Write;

//     bsp::console::console()
//         .write_fmt(args)
//         .expect("console print panic!!");
// }

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use screen::interface::Write;
    bsp::frame_buffer::screen()
        .write_fmt(args)
        .expect("screen print panic!!")
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::print::_screen_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => {
        $crate::screen_print!("{}\n", format_args!($($arg)*));
    };
}

// print an info with a newline
#[macro_export]
macro_rules! info {
    ($string:expr) => ({
        #[allow(unused_imports)]
        use crate::time::interface::TimeManager;

        let timestamp = $crate::time::time_manager().uptime();
        let timestamp_subsec_us = timestamp.subsec_micros();

        $crate::print::_print(format_args_nl!(
            concat!("[  {:>3}.{:03}{:03}] ", $string),
            timestamp.as_secs(),
            timestamp_subsec_us / 1_000,
            timestamp_subsec_us % 1_000
        ));
    });
    ($format_string:expr, $($arg:tt)*) => ({
        #[allow(unused_imports)]
        use crate::time::interface::TimeManager;

        let timestamp = $crate::time::time_manager().uptime();
        let timestamp_subsec_us = timestamp.subsec_micros();

        $crate::print::_print(format_args_nl!(
            concat!("[  {:>3}.{:03}{:03}] ", $format_string),
            timestamp.as_secs(),
            timestamp_subsec_us / 1_000,
            timestamp_subsec_us % 1_000,
            $($arg)*
        ));
    })
}

/// Prints a warning, with a newline.
#[macro_export]
macro_rules! warn {
    ($string:expr) => ({
        #[allow(unused_imports)]
        use crate::time::interface::TimeManager;

        let timestamp = $crate::time::time_manager().uptime();
        let timestamp_subsec_us = timestamp.subsec_micros();

        $crate::print::_print(format_args_nl!(
            concat!("[W {:>3}.{:03}{:03}] ", $string),
            timestamp.as_secs(),
            timestamp_subsec_us / 1_000,
            timestamp_subsec_us % 1_000
        ));
    });
    ($format_string:expr, $($arg:tt)*) => ({
        #[allow(unused_imports)]
        use crate::time::interface::TimeManager;

        let timestamp = $crate::time::time_manager().uptime();
        let timestamp_subsec_us = timestamp.subsec_micros();

        $crate::print::_print(format_args_nl!(
            concat!("[W {:>3}.{:03}{:03}] ", $format_string),
            timestamp.as_secs(),
            timestamp_subsec_us / 1_000,
            timestamp_subsec_us % 1_000,
            $($arg)*
        ));
    })
}

#[doc(hidden)]
pub fn _buffer_print(args: fmt::Arguments) {
    use screen::interface::Write;
    bsp::frame_buffer::screen()
        .write_fmt(args)
        .expect("screen print panic!!")
}

#[macro_export]
macro_rules! buffer_print {
    ($($arg:tt)*) => ($crate::print::_buffer_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! buffer_println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => {
        $crate::buffer_print!("{}\n", format_args!($($arg)*));
    };
}
