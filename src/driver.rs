//--------------------------------------------------------------------------------------------------
// Private Definitions
//--------------------------------------------------------------------------------------------------

// Driver interfaces
pub mod interface {
    // Device driver functions
    pub trait DeviceDriver {
        // retrun a compatibility string for indetifying the driver
        fn compatible(&self) -> &'static str;

        // called by the kernel to bring up the device
        unsafe fn init(&self) -> Result<(), &'static str> {
            Ok(())
        }

        fn register_and_enable_irq_handler(
            &'static self,
        ) -> Result<(), &'static str> {
            Ok(())
        }
    }

    // Device driver managment functions
    pub trait DriverManager {
        // return a slice of references to all `BSP`-instantiated drivers
        fn all_device_drivers(&self) -> &[&'static (dyn DeviceDriver + Sync)];

        // Initialization code that runs after driver init
        fn post_device_driver_init(&self);
    }
}
