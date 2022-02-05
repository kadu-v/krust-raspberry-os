use super::map as memory_map;
use crate::memory::mmu::*;
use core::ops::RangeInclusive;

//--------------------------------------------------------------------------------------------------
// Public Definitions
//--------------------------------------------------------------------------------------------------

// BSPで定義されているカーネル空間のアドレス
pub type KernelAddrSpace = AddressSpace<{ memory_map::END_INCLUSIVE + 1 }>;

const NUM_MEM_RANGES: usize = 3;

// 仮想アドレスレイアウト
pub static LAYOUT: KernelVirtualLayout<NUM_MEM_RANGES> = KernelVirtualLayout::new(
    memory_map::END_INCLUSIVE,
    [
        TranslationDescriptor {
            name: "Kernel code and R0 data",
            virtual_range: code_range_inclusive,
            physical_range_translation: Translation::Identity,
            attribute_fields: AttributeFields {
                mem_attributes: MemAttributes::CacheableDRAM,
                acc_perms: AccessPermissions::ReadOnly,
                execute_never: false,
            },
        },
        TranslationDescriptor {
            name: "Remapped Device MMIO",
            virtual_range: remmapped_mmio_range_inclusive,
            physical_range_translation: Translation::Offset(memory_map::mmio::START + 0x20_0000),
            attribute_fields: AttributeFields {
                mem_attributes: MemAttributes::Device,
                acc_perms: AccessPermissions::ReadWrite,
                execute_never: true,
            },
        },
        TranslationDescriptor {
            name: "Device MMIO",
            virtual_range: mmio_range_inclusive,
            physical_range_translation: Translation::Identity,
            attribute_fields: AttributeFields {
                mem_attributes: MemAttributes::Device,
                acc_perms: AccessPermissions::ReadWrite,
                execute_never: true,
            },
        },
    ],
);

//--------------------------------------------------------------------------------------------------
// Private Code
//--------------------------------------------------------------------------------------------------

fn code_range_inclusive() -> RangeInclusive<usize> {
    #[allow(clippy::range_minus_one)]
    RangeInclusive::new(super::code_start(), super::code_end_exclusive() - 1)
}

fn remmapped_mmio_range_inclusive() -> RangeInclusive<usize> {
    RangeInclusive::new(0x1FFF_0000, 0x1FFF_FFFF)
}

fn mmio_range_inclusive() -> RangeInclusive<usize> {
    RangeInclusive::new(memory_map::mmio::START, memory_map::mmio::END_INCLUSIVE)
}

//--------------------------------------------------------------------------------------------------
// Private Code
//--------------------------------------------------------------------------------------------------
pub fn virt_mem_layout() -> &'static KernelVirtualLayout<NUM_MEM_RANGES> {
    &LAYOUT
}
