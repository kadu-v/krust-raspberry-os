#[cfg(target_arch = "aarch64")]
#[path = "../_arch/aarch64/exception/asynchronous.rs"]
mod arch_asynchronous;

use crate::bsp;
use core::{fmt, marker::PhantomData};

//--------------------------------------------------------------------------------------------------
// Architectural Public Reexports
//--------------------------------------------------------------------------------------------------
pub use arch_asynchronous::{
    is_local_irq_masked, local_irq_mask, local_irq_mask_save,
    local_irq_restore, local_irq_unmask, print_state,
};

//--------------------------------------------------------------------------------------------------
// Public Definitions
//--------------------------------------------------------------------------------------------------
#[derive(Clone, Copy)]
pub struct IRQDescriptor {
    pub name: &'static str,
    pub handler: &'static (dyn interface::IRQHandler + Sync),
}

#[derive(Clone, Copy)]
pub struct IRQContext<'irq_context> {
    _0: PhantomData<&'irq_context ()>,
}

pub mod interface {
    pub trait IRQHandler {
        fn handle(&self) -> Result<(), &'static str>;
    }

    // IRQ management functions
    pub trait IRQManager {
        type IRQNumberType;
        fn register_handler(
            &self,
            irq_number: Self::IRQNumberType,
            descriptor: super::IRQDescriptor,
        ) -> Result<(), &'static str>;

        fn enable(&self, irq_number: Self::IRQNumberType);

        #[allow(clippy::trivially_copy_pass_by_ref)]
        fn handle_pending_irqs<'irq_context>(
            &'irq_context self,
            ic: &super::IRQContext<'irq_context>,
        );

        fn print_handler(&self);
    }
}

#[derive(Clone, Copy)]

pub struct IRQNumber<const MAX_INCLUSIVE: usize>(usize);

//--------------------------------------------------------------------------------------------------
// Public Code
//--------------------------------------------------------------------------------------------------
impl<'irq_context> IRQContext<'irq_context> {
    #[inline(always)]
    pub unsafe fn new() -> Self {
        IRQContext { _0: PhantomData }
    }
}

impl<const MAX_INCLUSIVE: usize> IRQNumber<{ MAX_INCLUSIVE }> {
    pub const fn new(number: usize) -> Self {
        assert!(number <= MAX_INCLUSIVE);
        Self(number)
    }

    pub const fn get(self) -> usize {
        self.0
    }
}

impl<const MAX_INCLUSIVE: usize> fmt::Display for IRQNumber<{ MAX_INCLUSIVE }> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub fn exec_with_irq_masked<T>(f: impl FnOnce() -> T) -> T {
    let saved = local_irq_mask_save();
    let ret = f();
    local_irq_restore(saved);

    ret
}

pub fn irq_manager(
) -> &'static dyn interface::IRQManager<IRQNumberType = bsp::driver::IRQNumber>
{
    bsp::exception::asynchronous::irq_manager()
}
