use crate::{
    bsp::device_driver::common::MMIODerefWrapper, state, synchronization,
    synchronization::IRQSafeNullLock,
};

use tock_registers::{
    interfaces::{Readable, Writeable},
    register_bitfields, register_structs,
    registers::{ReadOnly, ReadWrite},
};

//--------------------------------------------------------------------------------------------------
// Private Definitions
//--------------------------------------------------------------------------------------------------
register_bitfields! {
    u32,
    CTLR [
        Enable OFFSET(0) NUMBITS(1) []
    ],

    TYPER [
        ITLinesNumber OFFSET(0) NUMBITS(5) []
    ],

    ITARGETSR [
        Offset3 OFFSET(24) NUMBITS(8)[],
        Offset2 OFFSET(16) NUMBITS(8)[],
        Offset1 OFFSET(8) NUMBITS(8)[],
        Offset0 OFFSET(0) NUMBITS(8)[],
    ]
}

register_structs! {
    #[allow(non_snake_case)]
    SharedRegisterBlock {
        (0x000 => CTLR: ReadWrite<u32, CTLR::Register>),
        (0x004 => TYPER: ReadOnly<u32, TYPER::Register>),
        (0x008 => _reserved1),
        (0x104 => ISENABLER: [ReadWrite<u32>; 31]),
        (0x108 => _reserved2),
        (0x820 => ITARGETSR: [ReadWrite<u32, ITARGETSR::Register>; 248]),
        (0x824 => @END),
    }
}

register_structs! {
    #[allow(non_snake_case)]
    BankedRegisterBlock {
        (0x000 => _reserved1),
        (0x100 => ISENABLER: ReadWrite<u32>),
        (0x104 => _reserved2),
        (0x800 => ITARGETSR: [ReadOnly<u32, ITARGETSR::Register>; 8]),
        (0x804 => @END),
    }
}

type SharedRegisters = MMIODerefWrapper<SharedRegisterBlock>;

type BankedRegisters = MMIODerefWrapper<BankedRegisterBlock>;
//--------------------------------------------------------------------------------------------------
// Public Definitions
//--------------------------------------------------------------------------------------------------

/// Representation of the GIC Distributor.
pub struct GICD {
    /// Access to shared registers is guarded with a lock.
    shared_registers: IRQSafeNullLock<SharedRegisters>,

    /// Access to banked registers is unguarded.
    banked_registers: BankedRegisters,
}

//--------------------------------------------------------------------------------------------------
// Private Code
//--------------------------------------------------------------------------------------------------

impl SharedRegisters {
    /// Return the number of IRQs that this HW implements.
    #[inline(always)]
    fn num_irqs(&mut self) -> usize {
        // Query number of implemented IRQs.
        //
        // Refer to GICv2 Architecture Specification, Section 4.3.2.
        ((self.TYPER.read(TYPER::ITLinesNumber) as usize) + 1) * 32
    }

    /// Return a slice of the implemented ITARGETSR.
    #[inline(always)]
    fn implemented_itargets_slice(
        &mut self,
    ) -> &[ReadWrite<u32, ITARGETSR::Register>] {
        assert!(self.num_irqs() >= 36);

        // Calculate the max index of the shared ITARGETSR array.
        //
        // The first 32 IRQs are private, so not included in `shared_registers`. Each ITARGETS
        // register has four entries, so shift right by two. Subtract one because we start
        // counting at zero.
        let spi_itargetsr_max_index = ((self.num_irqs() - 32) >> 2) - 1;

        // Rust automatically inserts slice range sanity check, i.e. max >= min.
        &self.ITARGETSR[0..spi_itargetsr_max_index]
    }
}

//--------------------------------------------------------------------------------------------------
// Public Code
//--------------------------------------------------------------------------------------------------
use synchronization::interface::Mutex;

impl GICD {
    pub const unsafe fn new(mmio_start_addr: usize) -> Self {
        Self {
            shared_registers: IRQSafeNullLock::new(SharedRegisters::new(
                mmio_start_addr,
            )),
            banked_registers: BankedRegisters::new(mmio_start_addr),
        }
    }

    fn local_gic_target_mask(&self) -> u32 {
        self.banked_registers.ITARGETSR[0].read(ITARGETSR::Offset0)
    }

    pub fn boot_core_init(&self) {
        assert!(
            state::state_manager().is_init(),
            "Only allowed during kernel init phase"
        );

        let mask = self.local_gic_target_mask();

        self.shared_registers.lock(|regs| {
            for i in regs.implemented_itargets_slice().iter() {
                i.write(
                    ITARGETSR::Offset3.val(mask)
                        + ITARGETSR::Offset2.val(mask)
                        + ITARGETSR::Offset1.val(mask)
                        + ITARGETSR::Offset0.val(mask),
                );
            }

            regs.CTLR.write(CTLR::Enable::SET);
        });
    }

    /// Enable an interrupt.
    pub fn enable(&self, irq_num: super::IRQNumber) {
        let irq_num = irq_num.get();

        let enable_reg_index = irq_num >> 5;
        let enable_bit: u32 = 1u32 << (irq_num % 32);

        match irq_num {
            0..=31 => {
                let enable_reg = &self.banked_registers.ISENABLER;
                enable_reg.set(enable_reg.get() | enable_bit);
            }
            _ => {
                let enable_reg_index_shared = enable_reg_index - 1;

                self.shared_registers.lock(|regs| {
                    let enable_reg = &regs.ISENABLER[enable_reg_index_shared];
                    enable_reg.set(enable_reg.get() | enable_bit);
                });
            }
        }
    }
}
