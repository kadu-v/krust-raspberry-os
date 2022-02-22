use crate::{
    bsp::device_driver::common::MMIODerefWrapper,
    console, cpu, driver, synchronization,
    synchronization::{interface::Mutex, NullLock},
};
use tock_registers::{
    interfaces::{Readable, Writeable},
    register_bitfields, register_structs,
    registers::{ReadOnly, ReadWrite, WriteOnly},
};

// Descriptions taken from
// raspberypi 3ap
// - https://github.com/raspberrypi/documentation/files/1888662/BCM2837-ARM-Peripherals.-.Revised.-.V2-1.pdf
// raspberypi 4b
// - https://datasheets.raspberrypi.org/bcm2711/bcm2711-peripherals.pdf

//--------------------------------------------------------------------------------------------------
// Private Definitions
//--------------------------------------------------------------------------------------------------

// PL011 UART registers
register_bitfields! {u32,
    RD [
        // actual address
        ADDR OFFSET(4) NUMBITS(28) [],

        // channel
        CH OFFSET(0) NUMBITS(4) [],
    ],

    RST [
        RD OFFSET(30) NUMBITS(2) [
            Full = 0b10,
            Empty = 0b01,
        ],
    ],

    WD [
        // actual address
        ADDR OFFSET(4) NUMBITS(28) [],

        // channel
        CH OFFSET(0) NUMBITS(4) [],
    ],

    WST [
        WD OFFSET(30) NUMBITS(2) [
            Full = 0b10,
            Empty = 0b01,
        ],
    ],
}

register_structs! {
    pub RegisterBlock {
        (0x00 => RD: ReadWrite<u32, RD::Register>),
        (0x18 => RST: ReadOnly<u32, RST::Register>),
        (0x20 => WD: ReadWrite<u32, WD::Register>),
        (0x38 => WST: ReadOnly<u32, WST::Register>),
        (0x42 => @END),
    }
}

type Registers = MMIODerefWrapper<RegisterBlock>;

#[derive(Debug, Clone, Copy)]
pub enum MailBoxError {
    NotAligned,
}

//--------------------------------------------------------------------------------------------------
// Public Definitions
//--------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, Copy)]
pub struct Messeage {
    channel: u8,
    data: u32,
}

pub struct MailBoxInner {
    registers: Registers,
}

pub struct MailBox {
    inner: NullLock<MailBoxInner>,
}

//--------------------------------------------------------------------------------------------------
// Public Code
//--------------------------------------------------------------------------------------------------
impl Messeage {
    // dataでアドレスを送るときは16byte境界にあラインされている必要がある
    pub const unsafe fn new(data: u32, channel: u8) -> Self {
        Self { data, channel }
    }
}

impl MailBoxInner {
    pub const unsafe fn new(mmio_start_addr: usize) -> Self {
        Self {
            registers: Registers::new(mmio_start_addr),
        }
    }

    pub unsafe fn read_mailbox(
        &mut self,
        ch: u8,
    ) -> Result<Messeage, MailBoxError> {
        loop {
            // busy loop until Read buffer is Full
            while self.registers.RST.matches_all(RST::RD::Empty) {}

            // return if channel of recieved data is valid
            if self.registers.RD.get() & 0x0F == ch as u32 {
                let data = self.registers.RD.get();
                let channel = ch;
                return Ok(Messeage { data, channel });
            }
        }
    }

    pub unsafe fn write_mailbox(
        &mut self,
        msg: &Messeage,
    ) -> Result<(), MailBoxError> {
        // ptr is align of 16
        if msg.data & 0x0f != 0 {
            return Err(MailBoxError::NotAligned);
        }

        // busy loop until Write status buffer is empty
        while self.registers.WST.matches_all(WST::WD::Full) {}

        // set mssage buffer address to Read regsiter of mailbox
        self.registers.WD.write(WD::ADDR.val(msg.data >> 4));

        // set channel to Read register of mailbox
        self.registers.WD.write(WD::CH.val(msg.channel as u32));

        // busy loop
        Ok(())
    }
}

impl MailBox {
    pub const unsafe fn new(mmio_start_addr: usize) -> Self {
        Self {
            inner: NullLock::new(MailBoxInner::new(mmio_start_addr)),
        }
    }

    pub unsafe fn read_mailbox(
        &mut self,
        ch: u8,
    ) -> Result<Messeage, MailBoxError> {
        self.inner.lock(|inner| inner.read_mailbox(ch))
    }

    pub unsafe fn write_mailbox(
        &mut self,
        msg: &Messeage,
    ) -> Result<(), MailBoxError> {
        self.inner.lock(|inner| inner.write_mailbox(msg))
    }
}

//------------------------------------------------------------------------------
// Global Instance
//------------------------------------------------------------------------------
