pub mod device_driver;

#[cfg(any(feature = "bsp_rpi3ap", feature = "bsp_rpi3", feature = "bsp_rpi4"))]
mod raspberrypi;

#[cfg(any(
    feature = "bsp_rpi3ap",
    feature = "bsp_rpi3",
    feature = "bsp_rpi4"
))]
pub use raspberrypi::*;
