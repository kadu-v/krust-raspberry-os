#[cfg(feature = "bsp_rpi3")]
mod bcm2xx_interrupt_controller;
mod bcm2xxx_gpio;
mod bcm2xxx_pl011_uart;

#[cfg(feature = "bsp_rpi3")]
pub use bcm2xx_interrupt_controller::*;
pub use bcm2xxx_gpio::*;
pub use bcm2xxx_pl011_uart::*;
