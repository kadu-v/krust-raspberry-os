use crate::{
    bsp::{device_driver::common::MMIODerefWrapper, memory},
    console, cpu, driver, print, println, synchronization,
    synchronization::{interface::Mutex, NullLock},
};
use tock_registers::{
    interfaces::{ReadWriteable, Readable, Writeable},
    register_bitfields, register_structs,
    registers::{InMemoryRegister, ReadOnly, ReadWrite, WriteOnly},
};

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
    RD [
        // actual address
        DATA OFFSET(4) NUMBITS(32) [],
    ],

    ST [
        RD OFFSET(30) NUMBITS(2) [
            Full = 0b10,
            Empty = 0b01,
        ],
    ],

    WD [
        // actual address
        DATA OFFSET(4) NUMBITS(32) [],
    ],
}

register_structs! {
    #[allow(non_snake_case)]
    pub RegisterBlock {
        (0x00 => RD: InMemoryRegister<u32, RD::Register>),
        (0x04 => _reserved1),
        (0x18 => ST: InMemoryRegister<u32, ST::Register>),
        (0x1c => _reserved2),
        (0x20 => WD: InMemoryRegister<u32, WD::Register>),
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

#[derive(Debug, Clone, Copy)]
pub struct Messeage {
    pub channel: u8,
    pub data: u32,
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
        &self,
        ch: u8,
    ) -> Result<Messeage, MailBoxError> {
        println!("Status register {:x}", self.registers.ST.get());
        loop {
            // busy loop until Read buffer is Full
            while self.registers.ST.matches_all(ST::RD::Empty) {
                cpu::nop();
            }
            // println!("{:32b}, {}", self.registers.RD.get(), ch);
            // return if channel of recieved data is valid
            if self.registers.RD.read(RD::DATA) & 0x0F == ch as u32 {
                let data = self.registers.RD.read(RD::DATA) >> 4;
                let channel = ch;
                println!("end read mailbox: {:x}, {:x}", data, channel);
                return Ok(Messeage { data, channel });
            }
        }
    }

    pub unsafe fn write_mailbox(
        &self,
        msg: &Messeage,
    ) -> Result<(), MailBoxError> {
        // ptr is align of 16
        if msg.data & 0x0F != 0 {
            return Err(MailBoxError::NotAligned);
        }

        // busy loop until Write status buffer is empty
        while self.registers.ST.matches_all(ST::RD::Full) {
            cpu::nop();
        }

        println!("{:x}", msg.data + msg.channel as u32);
        self.registers.WD.write(WD::DATA.val(1 as u32));
        let ptr = (0x0000_B880 + 0x3F00_0000 + 0x00) as *mut u32;
        println!("raw RD: {:x}", core::ptr::read_volatile(ptr));
        let ptr = (0x0000_B880 + 0x3F00_0000 + 0x18) as *mut u32;
        println!("raw ST: {:x}", core::ptr::read_volatile(ptr));
        let ptr = (0x0000_B880 + 0x3F00_0000 + 0x20) as *mut u32;
        println!("raw WD: {:x}", core::ptr::read_volatile(ptr));

        println!("raw write");
        // set mssage buffer address to Read regsiter of mailbox
        let ptr = (0x0000_B880 + 0x3F00_0000 + 0x20) as *mut u32;
        core::ptr::write_volatile(ptr, 1);

        let ptr = (0x0000_B880 + 0x3F00_0000 + 0x00) as *mut u32;
        println!("raw RD: {:x}", core::ptr::read_volatile(ptr));
        let ptr = (0x0000_B880 + 0x3F00_0000 + 0x18) as *mut u32;
        println!("raw ST: {:x}", core::ptr::read_volatile(ptr));
        let ptr = (0x0000_B880 + 0x3F00_0000 + 0x20) as *mut u32;
        println!("raw WD: {:x}", core::ptr::read_volatile(ptr));

        println!("{}", msg.channel);
        // set channel to Read register of mailbox

        let ptr = (0x0000_B880 + 0x3F00_0000 + 0x00) as *mut u32;
        println!("raw RD: {:x}", core::ptr::read_volatile(ptr));
        let ptr = (0x0000_B880 + 0x3F00_0000 + 0x18) as *mut u32;
        println!("raw ST: {:x}", core::ptr::read_volatile(ptr));
        let ptr = (0x0000_B880 + 0x3F00_0000 + 0x20) as *mut u32;
        println!("raw WD: {:x}", core::ptr::read_volatile(ptr));

        cortex_a::asm::barrier::dsb(cortex_a::asm::barrier::SY);
        println!("end write mailbox: {:x}", self.registers.WD.get());
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
        &self,
        ch: u8,
    ) -> Result<Messeage, MailBoxError> {
        self.inner.lock(|inner| inner.read_mailbox(ch))
    }

    pub unsafe fn write_mailbox(
        &self,
        msg: &Messeage,
    ) -> Result<(), MailBoxError> {
        self.inner.lock(|inner| inner.write_mailbox(msg))
    }
}
