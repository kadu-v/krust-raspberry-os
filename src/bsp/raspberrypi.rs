pub mod console;
pub mod cpu;
pub mod driver;
pub mod exception;
pub mod frame_buffer;
pub mod mailbox;
pub mod memory;
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
