use crate::{
    bsp::device_driver::common::MMIODerefWrapper,
    cpu,
    synchronization::{interface::Mutex, IRQSafeNullLock},
};

use crate::print;
use tock_registers::{
    interfaces::{Readable, Writeable},
    register_bitfields, register_structs,
    registers::{InMemoryRegister, ReadOnly},
};
use volatile::Volatile;

// Descriptions taken from
// raspberypi 3ap
// - https://github.com/raspberrypi/documentation/files/1888662/BCM2837-ARM-Peripherals.-.Revised.-.V2-1.pdf
// raspberypi 4b
// - https://datasheets.raspberrypi.org/bcm2711/bcm2711-peripherals.pdf
// https://elinux.org/RPi_Framebuffer

//--------------------------------------------------------------------------------------------------
// Private Definitions
//--------------------------------------------------------------------------------------------------

// PL011 UART registers
register_bitfields! {u32,
    STATUS [
        EMPTY OFFSET(30) NUMBITS(1) [],
        FULL OFFSET(31) NUMBITS(1) []
    ],
}
// 自動でフィールドのと型の整合性を計算して、padding?するので注意
register_structs! {
    #[allow(non_snake_case)]
    pub RegisterBlock {
        (0x00 => READ: InMemoryRegister<u32>),
        (0x04 => _reserved1),
        (0x18 => STATUS: ReadOnly<u32, STATUS::Register>),
        (0x1c => _reserved2),
        (0x20 => WRITE: InMemoryRegister<u32>),
        (0x24 => @END),
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

#[derive(Debug, Clone)]
#[repr(C, align(16))]
pub struct Messege {
    pub data: [Volatile<u32>; 36],
    pub channel: u32,
}

pub struct MailBoxInner {
    registers: Registers,
}

pub struct MailBox {
    inner: IRQSafeNullLock<MailBoxInner>,
}

//--------------------------------------------------------------------------------------------------
// Public Code
//--------------------------------------------------------------------------------------------------
impl Messege {
    // dataでアドレスを送るときは16byte境界にあラインされている必要がある
    pub unsafe fn new(channel: u32) -> Self {
        let data = [
            Volatile::new(0u32),
            Volatile::new(0u32),
            Volatile::new(0u32),
            Volatile::new(0u32),
            Volatile::new(0u32),
            Volatile::new(0u32),
            Volatile::new(0u32),
            Volatile::new(0u32),
            Volatile::new(0u32),
            Volatile::new(0u32),
            Volatile::new(0u32),
            Volatile::new(0u32),
            Volatile::new(0u32),
            Volatile::new(0u32),
            Volatile::new(0u32),
            Volatile::new(0u32),
            Volatile::new(0u32),
            Volatile::new(0u32),
            Volatile::new(0u32),
            Volatile::new(0u32),
            Volatile::new(0u32),
            Volatile::new(0u32),
            Volatile::new(0u32),
            Volatile::new(0u32),
            Volatile::new(0u32),
            Volatile::new(0u32),
            Volatile::new(0u32),
            Volatile::new(0u32),
            Volatile::new(0u32),
            Volatile::new(0u32),
            Volatile::new(0u32),
            Volatile::new(0u32),
            Volatile::new(0u32),
            Volatile::new(0u32),
            Volatile::new(0u32),
            Volatile::new(0u32),
        ];
        Self {
            data: data,
            channel,
        }
    }
}

impl MailBoxInner {
    pub const unsafe fn new(mmio_start_addr: usize) -> Self {
        Self {
            registers: Registers::new(mmio_start_addr),
        }
    }

    pub unsafe fn mailbox_call(
        &self,
        msg: &mut Messege,
    ) -> Result<(), MailBoxError> {
        let ptr = msg.data.as_ptr() as u32;

        // Check alignment
        if ptr & 0x0F != 0 {
            return Err(MailBoxError::NotAligned);
        }

        // mailbox interface
        let data = (ptr | msg.channel) + 0xC000_0000;
        let response = 0x8000_0000;

        // wait until mailbox is empty
        while self.registers.STATUS.matches_all(STATUS::FULL::SET) {
            // wait here
        }

        // send a message via mailbox
        self.registers.WRITE.set(data);

        // wait until messeage is sended
        loop {
            while self.registers.STATUS.matches_all(STATUS::EMPTY::SET) {
                // wait here
            }

            let received_data = self.registers.READ.get();

            if received_data == data {
                // この呼び出しがないとmsg.data[1]の値が最適化で定数値にされる？
                if msg.data[1].read() == response {
                    return Ok(());
                }
            }
        }
    }
}

impl MailBox {
    pub const unsafe fn new(mmio_start_addr: usize) -> Self {
        Self {
            inner: IRQSafeNullLock::new(MailBoxInner::new(mmio_start_addr)),
        }
    }

    pub unsafe fn mailbox_call(
        &self,
        msg: &mut Messege,
    ) -> Result<(), MailBoxError> {
        self.inner.lock(|mbox| mbox.mailbox_call(msg))
    }
}
