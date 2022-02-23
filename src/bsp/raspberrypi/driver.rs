use crate::driver::{self, interface::DeviceDriver};

//--------------------------------------------------------------------------------------------------
// Private Definitions
//--------------------------------------------------------------------------------------------------

// Device Driver Manager type
struct BSPDriverManager {
    device_drivers: [&'static (dyn DeviceDriver + Sync); 3],
}

//--------------------------------------------------------------------------------------------------
// Global instaces
//--------------------------------------------------------------------------------------------------

static BSP_DRIVER_MANAGER: BSPDriverManager = BSPDriverManager {
    device_drivers: [&super::GPIO, &super::PL011_UART, &super::FRAMEBUFFER],
};

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
        super::GPIO.map_pl011_uart();
    }
}
