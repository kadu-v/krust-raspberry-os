#[cfg(any(feature = "bsp_rpi3ap"))]
mod raspberrypi;

#[cfg(any(feature = "bsp_rpi3ap"))]
pub use raspberrypi::*;
