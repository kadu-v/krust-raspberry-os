use super::{exception, frame_buffer, memory::map::mmio};
use crate::{bsp::device_driver, driver};
pub use device_driver::IRQNumber;
use driver::interface::DeviceDriver;

//--------------------------------------------------------------------------------------------------
// Private Definitions
//--------------------------------------------------------------------------------------------------

// Device Driver Manager type
struct BSPDriverManager {
    device_drivers: [&'static (dyn DeviceDriver + Sync); 4],
}

//--------------------------------------------------------------------------------------------------
// Global instaces
//--------------------------------------------------------------------------------------------------
pub(super) static PL011_UART: device_driver::PL011Uart = unsafe {
    device_driver::PL011Uart::new(
        mmio::PL011_UART_START,
        exception::asynchronous::irq_map::PL011_UART,
    )
};

static GPIO: device_driver::GPIO =
    unsafe { device_driver::GPIO::new(mmio::GPIO_START) };

#[cfg(feature = "bsp_rpi3")]
pub(super) static INTERRUPT_CONTROLLER: device_driver::InterruptController = unsafe {
    device_driver::InterruptController::new(
        mmio::PERIPHERAL_INTERRUPT_CONTROLLER_START,
    )
};

#[cfg(feature = "bsp_rpi4")]
pub(super) static INTERRUPT_CONTROLLER: device_driver::GICv2 =
    unsafe { device_driver::GICv2::new(mmio::GICD_START, mmio::GICC_START) };

pub(super) static FRAMEBUFFER: frame_buffer::FrameBuffer =
    frame_buffer::FrameBuffer::new();

static BSP_DRIVER_MANAGER: BSPDriverManager = BSPDriverManager {
    device_drivers: [&PL011_UART, &GPIO, &FRAMEBUFFER, &INTERRUPT_CONTROLLER],
};

pub(super) static MAILBOX: super::mailbox::MailBox =
    unsafe { super::mailbox::MailBox::new(mmio::MAILBOX_START) };

//--------------------------------------------------------------------------------------------------
// Pubilic Code
//--------------------------------------------------------------------------------------------------
// Return a refrence to the driver manager
pub fn driver_manager() -> &'static impl driver::interface::DriverManager {
    &BSP_DRIVER_MANAGER
}

//------------------------------------------------------------------------------
// OS Interface Code
//------------------------------------------------------------------------------

impl driver::interface::DriverManager for BSPDriverManager {
    fn all_device_drivers(&self) -> &[&'static (dyn DeviceDriver + Sync)] {
        &self.device_drivers[..]
    }

    fn post_device_driver_init(&self) {
        GPIO.map_pl011_uart();
    }
}
