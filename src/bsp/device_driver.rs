//! Device driver.

#[cfg(any(feature = "bsp_rpi3", feature = "bsp_rpi3ap", feature = "bsp_rpi4"))]
mod bcm;
mod common;
mod mailbox;

#[cfg(any(
    feature = "bsp_rpi3",
    feature = "bsp_rpi3ap",
    feature = "bsp_rpi4"
))]
pub use bcm::*;
pub use mailbox::*;
