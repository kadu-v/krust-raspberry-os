mod gicc;
mod gicd;

use crate::{
    bsp, cpu, driver, exception, synchronization,
    synchronization::InitStateLock,
};

//--------------------------------------------------------------------------------------------------
// Private Definitions
//--------------------------------------------------------------------------------------------------

type HadlerTable =
    [Option<exception::asynchronous::IRQDescriptor>; GICv2::NUM_IRQS];

//--------------------------------------------------------------------------------------------------
// Public Definitions
//--------------------------------------------------------------------------------------------------
pub type IRQNumber =
    exception::asynchronous::IRQNumber<{ GICv2::MAX_IRQ_NUMBER }>;

pub struct GICv2 {
    gicd: gicd::GICD,

    gicc: gicc::GICC,

    handler_table: InitStateLock<HadlerTable>,
}

//--------------------------------------------------------------------------------------------------
// Public Code
//--------------------------------------------------------------------------------------------------

impl GICv2 {
    const MAX_IRQ_NUMBER: usize = 300;
    const NUM_IRQS: usize = Self::MAX_IRQ_NUMBER + 1;

    pub const COMPATIBLE: &'static str =
        "GICv2 (ARM Generic Interrupt Constroller v2)";

    pub const unsafe fn new(
        gicd_mmio_start_addr: usize,
        gicc_mmio_start_addr: usize,
    ) -> Self {
        Self {
            gicd: gicd::GICD::new(gicd_mmio_start_addr),
            gicc: gicc::GICC::new(gicc_mmio_start_addr),
            handler_table: InitStateLock::new([None; Self::NUM_IRQS]),
        }
    }
}

//------------------------------------------------------------------------------
// OS Interface Code
//------------------------------------------------------------------------------
use synchronization::interface::ReadWriteEx;

impl driver::interface::DeviceDriver for GICv2 {
    fn compatible(&self) -> &'static str {
        Self::COMPATIBLE
    }

    unsafe fn init(&self) -> Result<(), &'static str> {
        if bsp::cpu::BOOT_CORE_ID == cpu::smp::core_id() {
            self.gicd.boot_core_init();
        }

        self.gicc.priority_accept_all();
        self.gicc.enable();

        Ok(())
    }
}

impl exception::asynchronous::interface::IRQManager for GICv2 {
    type IRQNumberType = IRQNumber;

    fn register_handler(
        &self,
        irq_number: Self::IRQNumberType,
        descriptor: exception::asynchronous::IRQDescriptor,
    ) -> Result<(), &'static str> {
        self.handler_table.write(|table| {
            let irq_number = irq_number.get();

            if table[irq_number].is_some() {
                return Err("IRQ handler already registers");
            }

            table[irq_number] = Some(descriptor);

            Ok(())
        })
    }

    fn enable(&self, irq_number: Self::IRQNumberType) {
        self.gicd.enable(irq_number);
    }

    fn handle_pending_irqs<'irq_context>(
        &'irq_context self,
        ic: &exception::asynchronous::IRQContext<'irq_context>,
    ) {
        let irq_number = self.gicc.pending_irq_number(ic);

        if irq_number > GICv2::MAX_IRQ_NUMBER {
            return;
        }

        self.handler_table.read(|table| match table[irq_number] {
            None => panic!("No handler registered for IRQ {}", irq_number),
            Some(descriptor) => {
                descriptor.handler.handle().expect("Error handling IRQ")
            }
        });

        self.gicc.mark_completed(irq_number as u32, ic);
    }

    fn print_handler(&self) {
        use crate::info;

        info!("Peripheral handler");

        self.handler_table.read(|table| {
            for (i, opt) in table.iter().skip(32).enumerate() {
                if let Some(handler) = opt {
                    info!("{: >3}, {}", i + 32, handler.name);
                }
            }
        })
    }
}
