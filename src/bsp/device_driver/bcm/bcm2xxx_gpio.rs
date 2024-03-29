// GPIO Driver

use crate::{
    bsp::device_driver::common::MMIODerefWrapper,
    driver,
    synchronization::{interface::Mutex, IRQSafeNullLock},
};
use tock_registers::{
    interfaces::{ReadWriteable, Writeable},
    register_bitfields, register_structs,
    registers::ReadWrite,
};

// Descriptions taken from
// raspberypi 3ap
// - https://github.com/raspberrypi/documentation/files/1888662/BCM2837-ARM-Peripherals.-.Revised.-.V2-1.pdf
// raspberypi 4b
// - https://datasheets.raspberrypi.org/bcm2711/bcm2711-peripherals.pdf

//--------------------------------------------------------------------------------------------------
// Private Definitions
//--------------------------------------------------------------------------------------------------

// GPIO register

register_bitfields! {
    u32,

    GPFSEL1 [
        // Pin 15
        FSEL15 OFFSET(15) NUMBITS(3) [
            Input = 0b000,
            Output = 0b001,
            AltFunc0 = 0b100
        ],


        // Pin 14
        FSEL14 OFFSET(12) NUMBITS(3) [
            Input = 0b000,
            Output = 0b001,
            AltFunc0 = 0b100
        ]
    ],


    // BCM2837 Only
    GPPUD [
        PUD OFFSET(0) NUMBITS(2) [
            Off = 0b00,
            PullDown = 0b01,
            PullUp = 0b10
        ]
    ],


    GPPUDLK0 [
        // Pin15
        PUDLC15 OFFSET(15) NUMBITS(1) [
            NoEffect = 0,
            AssertClock = 1
        ],

        // Pin14
        PUDLC14 OFFSET(14) NUMBITS(1) [
            NoEffect = 0,
            AssertClock = 1
        ]
    ],

    // BCM2837 Only
    GPIO_PUP_CNTRL_REG0 [
        // Pin 15
        GPIO_PUP_PDN_CNTRL15 OFFSET(30) NUMBITS(2) [
            NoRegister = 0b00,
            PullUp = 0b01
        ],

        // Pin 14
        GPIO_PUP_PDN_CNTRL14 OFFSET(28) NUMBITS(2) [
            NoRegister = 0b00,
            PullUp = 0b01
        ]
    ]

}

register_structs! {
    #[allow(non_snake_case)]
    RegisterBlock {
        (0x00 => _reserved1),
        (0x04 => GPFSEL1: ReadWrite<u32, GPFSEL1::Register>),
        (0x08 => _reserved2),
        (0x94 => GPPUD: ReadWrite<u32, GPPUD::Register>),
        (0x98 => GPPUDLK0: ReadWrite<u32, GPPUDLK0::Register>),
        (0xE4 => GPIO_PUP_CNTRL_REG0: ReadWrite<u32, GPIO_PUP_CNTRL_REG0::Register>),
        (0xE8 => @END),
    }
}

type Registers = MMIODerefWrapper<RegisterBlock>;

//--------------------------------------------------------------------------------------------------
// Public Definitions
//--------------------------------------------------------------------------------------------------

pub struct GPIOInner {
    registers: Registers,
}

// Export the inner struct so that BSPs can use it for the panic handler
pub use GPIOInner as PanicGPIO;

// Representation of the GPIO HW
pub struct GPIO {
    inner: IRQSafeNullLock<GPIOInner>,
}

//--------------------------------------------------------------------------------------------------
// Public Code
//--------------------------------------------------------------------------------------------------

impl GPIOInner {
    // create an instance
    pub const unsafe fn new(mmio_star_addr: usize) -> Self {
        Self {
            registers: Registers::new(mmio_star_addr),
        }
    }

    // disable pull-up/down on pins 14 and 15
    #[cfg(any(feature = "bsp_rpi3", feature = "bsp_rpi3ap"))]
    fn disable_pud_14_15_bcm2837(&mut self) {
        use crate::{time, time::interface::TimeManager};
        use core::time::Duration;

        // The Linux 2837 GPIO 1 us between the steps
        const DELAY: Duration = Duration::from_micros(1);

        self.registers.GPPUD.write(GPPUD::PUD::Off);
        time::time_manager().spin_for(DELAY);

        self.registers.GPPUDLK0.write(
            GPPUDLK0::PUDLC15::AssertClock + GPPUDLK0::PUDLC14::AssertClock,
        );
        time::time_manager().spin_for(DELAY);

        self.registers.GPPUD.write(GPPUD::PUD::Off);
        self.registers.GPPUDLK0.set(0);
    }

    // disable pull-up/down on pins 14 and 15
    #[cfg(any(feature = "bsp_rpi4"))]
    fn disable_pud_14_15_bcm2711(&mut self) {
        self.registers.GPIO_PUP_CNTRL_REG0.write(
            GPIO_PUP_CNTRL_REG0::GPIO_PUP_PDN_CNTRL15::PullUp
                + GPIO_PUP_CNTRL_REG0::GPIO_PUP_PDN_CNTRL14::PullUp,
        )
    }

    // Map PL011 UART as standard output
    pub fn map_pl011_uart(&mut self) {
        self.registers
            .GPFSEL1
            .modify(GPFSEL1::FSEL14::AltFunc0 + GPFSEL1::FSEL14::AltFunc0);

        #[cfg(any(feature = "bsp_rpi3", feature = "bsp_rpi3ap"))]
        self.disable_pud_14_15_bcm2837();

        #[cfg(feature = "bsp_rpi4")]
        self.disable_pud_14_15_bcm2711();
    }
}

impl GPIO {
    // Create an instance
    pub const unsafe fn new(mmio_start_addr: usize) -> Self {
        Self {
            inner: IRQSafeNullLock::new(GPIOInner::new(mmio_start_addr)),
        }
    }

    // Concurency safe version of `GPIOInner.map_pl011_uart()`
    pub fn map_pl011_uart(&self) {
        self.inner.lock(|inner| inner.map_pl011_uart());
    }
}

//------------------------------------------------------------------------------
// OS Interface Code
//------------------------------------------------------------------------------
impl driver::interface::DeviceDriver for GPIO {
    fn compatible(&self) -> &'static str {
        "BCM GPIO"
    }
}
