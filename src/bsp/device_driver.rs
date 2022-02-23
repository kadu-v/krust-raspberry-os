//! Device driver.

#[cfg(any(feature = "bsp_rpi3", feature = "bsp_rpi3ap", feature = "bsp_rpi4"))]
mod bcm;
pub mod common;

#[cfg(any(
    feature = "bsp_rpi3",
    feature = "bsp_rpi3ap",
    feature = "bsp_rpi4"
))]
pub use bcm::*;
