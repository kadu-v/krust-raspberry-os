use super::map as memory_map;
// use crate::memmory::mmu::*;
use core::ops::RangeInclusive;

//--------------------------------------------------------------------------------------------------
// Public Definitions
//--------------------------------------------------------------------------------------------------

// BSPで定義されているカーネル空間のアドレス
// pub type KernelAddrSpace = AddressSpace<{ memory_map::END_INCLUSIVE + 1 }>;
