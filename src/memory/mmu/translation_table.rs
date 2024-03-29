#[cfg(target_arch = "aarch64")]
#[path = "../../_arch/aarch64/memory/mmu/translation_table.rs"]
mod arch_translation_table;

//--------------------------------------------------------------------------------------------------
// Architectural Public Reexports
//--------------------------------------------------------------------------------------------------
pub use arch_translation_table::KernelTranslationTable;
