use crate::{
    bsp::device_driver::common::MMIODerefWrapper, console, cpu, driver,
    synchronization, synchronization::NullLock,
};
use core::fmt;
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
register_bitfields! {
    u32,

    // FLAG Register
    FR [
        // Transmit FIFO emtpy
        // The meaning of this bit depends on the state of the FEN bit
        // in the Line Control Register, LCH_N
        TXFE OFFSET(7) NUMBITS(1) [],

        // Transmit FIFO full
        // The meaning of this bit deoends on the state of the FEN bit
        // in the LCH_N Register
        TXFF OFFSET(5) NUMBITS(1) [],

        // Recive FIFO empty
        // The meaning of this bit depends on the state of the FEN bit
        // in the LCR_H Register
        RXFE OFFSET(4)  NUMBITS(1) [],

        // Uart busy
        // If this bit is set to 1, the Uart is busy trasnmitting data
        // This bit remains set until comlete byte, including all the stop bits,
        // has been sent from the shift register

        BUSY OFFSET(3) NUMBITS(1) []
    ],

    // Integer bouad Rate Divisor
    IBRD [
        // The integer boud rate divisor
        BOUD_DIVINT OFFSET(0) NUMBITS(16) []
    ],

    // Fractional Baud Rate Divisor
    FBRD [
        //  The fractional baud rate divisor
        BOUD_DIVFRAC OFFSET(0) NUMBITS(6) []
    ],

    // Line Control Register
    LCR_H [
        #[allow(clippy::enum_variant_names)]
        WLEN OFFSET(5) NUMBITS(2) [
            FiveBit  = 0b00,
            SixBit   = 0b01,
            SevenBit = 0b10,
            EightBit = 0b11
        ],

        // Enable FIFOs
        // 0 = FIFOs are disabled (character mode) that is, the FIFOs become 1-byte-deep holding
        // registers.
        //
        // 1 = Transmit and receive FIFO buffers are enabled (FIFO mode).
        FEN OFFSET(4) NUMBITS(1) [
            FifosDisabled = 0,
            FifosEnabled  = 1
        ]

    ],

    // Control Register
    CR [
        // Receive enable
        RXE OFFSET(9) NUMBITS(1) [
            Disabled = 0,
            Enabled  = 1
        ],

        // Transmit enable
        TXE OFFSET(8) NUMBITS(1) [
            Disabled = 0,
            Enabled  = 1
        ],

        // Uart enable
        UARTEN OFFSET(0) NUMBITS(1) [
            Disabled = 0,
            Enabled  = 1
        ],
    ],


    // Integer Clear Register
    ICR [
        ALL OFFSET(0) NUMBITS(11) []
    ]
}

register_structs! {
    #[allow(non_snake_case)]
    pub RegisterBlock {
        (0x00 => DR: ReadWrite<u32>),
        (0x04 => _reserved1),
        (0x18 => FR: ReadOnly<u32, FR::Register>),
        (0x1c => _reserved2),
        (0x24 => IBRD: WriteOnly<u32, IBRD::Register>),
        (0x28 => FBRD: WriteOnly<u32, FBRD::Register>),
        (0x2c => LCR_H: WriteOnly<u32, LCR_H::Register>),
        (0x30 => CR: WriteOnly<u32, CR::Register>),
        (0x34 => _reserved3),
        (0x44 => ICR: WriteOnly<u32, ICR::Register>),
        (0x48 => @END),
    }
}

type Registers = MMIODerefWrapper<RegisterBlock>;

#[derive(PartialEq)]
enum BlockingMode {
    Blocking,
    NonBlocking,
}

//--------------------------------------------------------------------------------------------------
// Public Definitions
//--------------------------------------------------------------------------------------------------

pub struct PL011UartInner {
    registers: Registers,
    chars_written: usize,
    chars_read: usize,
}

// Export the inner struct so that BSPs can use it for the panic handler
pub use PL011UartInner as PanicUart;

// Representation of the Uart
pub struct PL011Uart {
    inner: NullLock<PL011UartInner>,
}

//--------------------------------------------------------------------------------------------------
// Public Code
//--------------------------------------------------------------------------------------------------

impl PL011UartInner {
    pub const unsafe fn new(mmio_start_addr: usize) -> Self {
        Self {
            registers: Registers::new(mmio_start_addr),
            chars_written: 0,
            chars_read: 0,
        }
    }

    pub fn init(&mut self) {
        self.flush();

        // Turn the UART off temporarily.
        self.registers.CR.set(0);
        // Clear all pending interrupts.
        self.registers.ICR.write(ICR::ALL::CLEAR);

        // Set the baud rate, 8N1 and FIFO enabled.
        self.registers.IBRD.write(IBRD::BOUD_DIVINT.val(3));
        self.registers.FBRD.write(FBRD::BOUD_DIVFRAC.val(16));
        self.registers
            .LCR_H
            .write(LCR_H::WLEN::EightBit + LCR_H::FEN::FifosEnabled);

        // Turn the Uart on
        self.registers
            .CR
            .write(CR::UARTEN::Enabled + CR::TXE::Enabled + CR::RXE::Enabled);
    }

    pub fn write_char(&mut self, c: char) {
        while self.registers.FR.matches_all(FR::TXFF::SET) {
            cpu::nop();
        }

        self.registers.DR.set(c as u32);

        self.chars_written += 1;
    }

    fn flush(&self) {
        while self.registers.FR.matches_all(FR::BUSY::SET) {
            cpu::nop();
        }
    }

    fn read_char_converting(
        &mut self,
        blocking_mode: BlockingMode,
    ) -> Option<char> {
        // 受信FIFOが空の場合
        if self.registers.FR.matches_all(FR::RXFE::SET) {
            // ノンブロッキングモードの場合は、即座にreturn
            if blocking_mode == BlockingMode::NonBlocking {
                return None;
            }
            // ブロッキングモードの場合は、charを受信するまで待つ
            while self.registers.FR.matches_all(FR::RXFE::SET) {
                cpu::nop();
            }
        }

        // charをFIFOから読み取る
        let mut ret = self.registers.DR.get() as u8 as char;

        if ret == '\r' {
            ret = '\n'
        }

        self.chars_read += 1;

        Some(ret)
    }
}

impl fmt::Write for PL011UartInner {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.chars() {
            self.write_char(c);
        }

        Ok(())
    }
}

impl PL011Uart {
    pub const unsafe fn new(mmio_start_addr: usize) -> Self {
        Self {
            inner: NullLock::new(PL011UartInner::new(mmio_start_addr)),
        }
    }
}

//------------------------------------------------------------------------------
// OS Interface Code
//------------------------------------------------------------------------------

use synchronization::interface::Mutex;

impl driver::interface::DeviceDriver for PL011Uart {
    fn compatible(&self) -> &'static str {
        "BCM PL011 UART"
    }

    unsafe fn init(&self) -> Result<(), &'static str> {
        self.inner.lock(|inner| inner.init());

        Ok(())
    }
}

impl console::interface::Write for PL011Uart {
    fn write_char(&self, c: char) {
        self.inner.lock(|inner| inner.write_char(c));
    }

    fn write_fmt(&self, args: fmt::Arguments) -> fmt::Result {
        self.inner.lock(|inner| fmt::Write::write_fmt(inner, args))
    }

    fn flush(&self) {
        self.inner.lock(|inner| inner.flush());
    }
}

impl console::interface::Read for PL011Uart {
    /// 受信FIFO(RX)から一文字を読み取る
    fn read_char(&self) -> char {
        self.inner.lock(|inner| {
            inner.read_char_converting(BlockingMode::Blocking).unwrap()
        })
    }

    /// 受信FIFO(RX)を空にする
    fn clear_rx(&self) {
        // FIFO の中身が空になるまで読み取る
        while self
            .inner
            .lock(|inner| inner.read_char_converting(BlockingMode::NonBlocking))
            .is_some()
        {}
    }
}

impl console::interface::Statistics for PL011Uart {
    fn chars_written(&self) -> usize {
        self.inner.lock(|inner| inner.chars_written)
    }

    fn chars_read(&self) -> usize {
        self.inner.lock(|inner| inner.chars_read)
    }
}
