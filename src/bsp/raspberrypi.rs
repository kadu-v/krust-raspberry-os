pub mod console;
pub mod cpu;
pub mod driver;
pub mod frame_buffer;
pub mod mailbox;
pub mod memory;

//--------------------------------------------------------------------------------------------------
// Global instances
//--------------------------------------------------------------------------------------------------

use super::device_driver;

static GPIO: device_driver::GPIO =
    unsafe { device_driver::GPIO::new(memory::map::mmio::GPIO_START) };

static PL011_UART: device_driver::PL011Uart = unsafe {
    device_driver::PL011Uart::new(memory::map::mmio::PL011_UART_START)
};

static MAILBOX: mailbox::MailBox =
    unsafe { mailbox::MailBox::new(memory::map::mmio::MAILBOX_START) };

static FRAMEBUFFER: frame_buffer::FrameBuffer =
    unsafe { frame_buffer::FrameBuffer::new() };

//--------------------------------------------------------------------------------------------------
// Public Code
//--------------------------------------------------------------------------------------------------
pub fn board_name() -> &'static str {
    #[cfg(feature = "bsp_rpi3")]
    {
        "Raspberry Pi 3"
    }

    #[cfg(feature = "bsp_rp3ap")]
    {
        "Rasberry Pi 3A+"
    }

    #[cfg(feature = "bsp_rpi4")]
    {
        "Raspberry Pi 4"
    }
}
