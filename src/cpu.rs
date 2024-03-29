#[cfg(target_arch = "aarch64")]
#[path = "_arch/aarch64/cpu.rs"]
mod arch_cpu;

mod boot;

pub mod smp;
//-------------------------------------------------------------------------------------------------
// Archtectural Public Reexports
//-------------------------------------------------------------------------------------------------
pub use arch_cpu::{nop, wait_forever};
