use super::memory;
use crate::bsp::device_driver;
use crate::console;
use crate::{synchronization, synchronization::NullLock};
use core::fmt;

//-------------------------------------------------------------------------------------------------
// Public Code
//-------------------------------------------------------------------------------------------------

// In case of a panic, the panic handler uses this function to take a last shot at printing
// something before the system is halted
pub unsafe fn panic_console_out() -> impl fmt::Write {
    let mut panic_gpio = device_driver::PanicGPIO::new(memory::map::mmio::GPIO_START);
    let mut panic_uart = device_driver::PanicUart::new(memory::map::mmio::PL011_UART_START);

    panic_gpio.map_pl011_uart();
    panic_uart.init();
    panic_uart
}

// Return a reference to the console
pub fn console() -> &'static impl console::interface::All {
    &super::PL011_UART
}
