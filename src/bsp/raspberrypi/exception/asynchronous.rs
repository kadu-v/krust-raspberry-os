use crate::{bsp, bsp::driver, exception};

//--------------------------------------------------------------------------------------------------
// Public Definitions
//--------------------------------------------------------------------------------------------------

#[cfg(feature = "bsp_rpi3")]
pub(in crate::bsp) mod irq_map {
    use super::bsp::device_driver::{IRQNumber, PeripheralIRQ};

    pub const PL011_UART: IRQNumber =
        IRQNumber::Peripheral(PeripheralIRQ::new(57));
}

#[cfg(feature = "bsp_rpi4")]
pub(in crate::bsp) mod irq_map {
    use super::bsp::device_driver::IRQNumber;

    pub const PL011_UART: IRQNumber = IRQNumber::new(153);
}

//--------------------------------------------------------------------------------------------------
// Public Code
//--------------------------------------------------------------------------------------------------

pub fn irq_manager(
) -> &'static impl exception::asynchronous::interface::IRQManager<
    IRQNumberType = bsp::device_driver::IRQNumber,
> {
    &driver::INTERRUPT_CONTROLLER
}
