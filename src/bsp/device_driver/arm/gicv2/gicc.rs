use crate::{bsp::device_driver::common::MMIODerefWrapper, exception};
use tock_registers::{
    interfaces::{Readable, Writeable},
    register_bitfields, register_structs,
    registers::ReadWrite,
};

register_bitfields! {
    u32,

    // CPU Interface Control Register
    CTLR [
        Enable OFFSET(0) NUMBITS(1) []
    ],

    // Interrupt Priority Mask Register
    PMR [
        Priority OFFSET(0) NUMBITS(8) []
    ],

    // Interrupt Acknoledge Register
    IAR [
        interruptID OFFSET(0) NUMBITS(10) []
    ],

    EOIR [
        EOIINITID OFFSET(0) NUMBITS(10) []
    ],
}

register_structs! {
    #[allow(non_snake_case)]
    pub RegisterBlock {
        (0x000 => CTLR: ReadWrite<u32, CTLR::Register>),
        (0x004 => PMR: ReadWrite<u32, PMR::Register>),
        (0x008 => _reserved),
        (0x00C => IAR: ReadWrite<u32, IAR::Register>),
        (0x010 => EOIR: ReadWrite<u32, EOIR::Register>),
        (0x014  => @END),
    }
}

type Registers = MMIODerefWrapper<RegisterBlock>;

//--------------------------------------------------------------------------------------------------
// Public Definitions
//--------------------------------------------------------------------------------------------------

// GIC CPU Interface
pub struct GICC {
    registers: Registers,
}

//--------------------------------------------------------------------------------------------------
// Public Code
//--------------------------------------------------------------------------------------------------
impl GICC {
    pub const unsafe fn new(mmio_start_addr: usize) -> Self {
        Self {
            registers: Registers::new(mmio_start_addr),
        }
    }

    pub fn priority_accept_all(&self) {
        self.registers.PMR.write(PMR::Priority.val(255));
    }

    pub fn enable(&self) {
        self.registers.CTLR.write(CTLR::Enable::SET);
    }

    #[allow(clippy::trivially_copy_pass_by_ref)]
    pub fn pending_irq_number<'irq_context>(
        &self,
        _ic: &exception::asynchronous::IRQContext<'irq_context>,
    ) -> usize {
        self.registers.IAR.read(IAR::interruptID) as usize
    }

    #[allow(clippy::trivially_copy_pass_by_ref)]
    pub fn mark_completed<'irq_context>(
        &self,
        irq_number: u32,
        _ic: &exception::asynchronous::IRQContext<'irq_context>,
    ) {
        self.registers.EOIR.write(EOIR::EOIINITID.val(irq_number));
    }
}
