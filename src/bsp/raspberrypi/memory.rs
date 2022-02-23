//BSP Memory Management.

// BSP Memory Management.
//
// The physical memory layout.
//
// The Raspberry's firmware copies the kernel binary to 0x8_0000. The preceding region will be used
// as the boot core's stack.
//
// +---------------------------------------+
// |                                       | 0x0
// |                                       |                                ^
// | Boot-core Stack                       |                                | stack
// |                                       |                                | growth
// |                                       |                                | direction
// +---------------------------------------+
// |                                       | code_start @ 0x8_0000
// | .text                                 |
// | .rodata                               |
// | .got                                  |
// |                                       |
// +---------------------------------------+
// |                                       | code_end_exclusive
// | .data                                 |
// | .bss                                  |
// |                                       |
// +---------------------------------------+
// |                                       |
// |                                       |

pub mod mmu;

use core::cell::UnsafeCell;

//--------------------------------------------------------------------------------------------------
// Private Definitions
//--------------------------------------------------------------------------------------------------

// Symbol from the linker script

extern "Rust" {
    static __code_start: UnsafeCell<()>;
    static __code_end_exclusive: UnsafeCell<()>;
}

//--------------------------------------------------------------------------------------------------
// Public Definitions
//--------------------------------------------------------------------------------------------------

#[rustfmt::skip]
pub(super) mod map {
    // End address *1 は2の冪乗でなければならない
    // rasp3とrasp4ではメモリの容量が異なるが、教育用にメモリの容量は4GiBに設定する．
    // したがって、rasp3では4GiBに近い容量のメモリを必要とするプログラムの場合にクラッシュする
    pub const END_INCLUSIVE:  usize  = 0xFFFF_FFFF;
    pub const GPIO_OFFSET:    usize  = 0x0020_0000;
    pub const UART_OFFSET:    usize  = 0x0020_1000;
    pub const MAILBOX_OFFSET: usize  = 0x0000_B880;

    #[cfg(feature = "bsp_rpi3")]
    pub mod mmio {
        use super::*;

        pub const START:            usize   =         0x3F00_0000;
        pub const GPIO_START:       usize   = START + GPIO_OFFSET;
        pub const PL011_UART_START: usize   = START + UART_OFFSET;
        pub const END_INCLUSIVE:    usize   =         0x4000_FFFF;
        pub const MAILBOX_START:    usize   = START + MAILBOX_OFFSET;
    }

    #[cfg(feature = "bsp_rpi4")]
    pub mod mmio {
        use super::*;

        pub const START:            usize   =         0xFE00_0000;
        pub const GPIO_START:       usize   = START + GPIO_OFFSET;
        pub const PL011_UART_START: usize   = START + UART_OFFSET;
        pub const END_INCLUSIVE:    usize   =         0xFF84_FFFF;
        pub const MAILBOX_START:    usize   = START + MAILBOX_OFFSET;
    }
}

// コードセグメントの開始アドレス
#[inline(always)]
fn code_start() -> usize {
    unsafe { __code_start.get() as usize }
}

// コードセグメントの排他的終端アドレス
#[inline(always)]
fn code_end_exclusive() -> usize {
    unsafe { __code_end_exclusive.get() as usize }
}
