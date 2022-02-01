#[cfg(target_arch = "aarch64")]
#[path = "../_arch/aarch64/memory/mmu.rs"]
mod arch_mmu;

mod translation_table;

use core::{fmt, ops::RangeInclusive};

//--------------------------------------------------------------------------------------------------
// Architectural Public Reexports
//--------------------------------------------------------------------------------------------------
// pub use arch_mmu::mmu;

use crate::console::interface::Write;

//--------------------------------------------------------------------------------------------------
// Public Reexports
//--------------------------------------------------------------------------------------------------

// MMU enable errors variants
#[derive(Debug)]
pub enum MMUEnableError {
    AlreadyEnabled,
    Other(&'static str),
}

// Memory Management interface
pub mod interface {
    use super::*;

    // MMU functions
    pub trait MMU {
        // カーネルの初期に呼び出される
        // BSPが提供する `virt_mem_layout()` から変換テーブルを取得し、
        // それぞれのMMUに対して install/activate を行う
        unsafe fn enable_mmu_and_chacing(&self) -> Result<(), MMUEnableError>;

        // MMUが利用可能なら、true
        // そのほかは false
        fn is_enabled(&self) -> bool;
    }
}

// 変換テーブルの大きさの設定
pub struct TranslationGranule<const GRANULE_SIZE: usize>;

// アドレス空間の設定
pub struct AddressSpace<const AS_SIZE: usize>;

// アーキテクチャ非依存の変換の型
#[allow(dead_code)]
#[derive(Copy, Clone)]
pub enum Translation {
    Identity,
    Offset(usize),
}

// アーキテクチャ非依存のメモリアトリビュート
#[derive(Copy, Clone)]
pub enum MemAttributes {
    CacheableDRAM,
    Device,
}

// アーキテクチャ非依存のアクセスパーミッション
#[derive(Copy, Clone)]
pub enum AcessPermissons {
    ReadOnly,
    ReadWrite,
}

// メモリのアトリビュート
#[derive(Copy, Clone)]
pub struct AttributeFields {
    pub mem_attributes: MemAttributes,
    pub acc_perms: AcessPermissons,
    pub execute_never: bool,
}

// アーキテクチャ非依存のディスクリプタ
pub struct TranslationDescriptor {
    pub name: &'static str,
    pub virtual_range: fn() -> RangeInclusive<usize>,
    pub physical_range_translation: Translation,
    pub attribute_fields: AttributeFields,
}

// カーネルの仮想メモリのための型
pub struct KernelVirtualLayout<const NUM_SPECIAL_RANGES: usize> {
    //　アドレス空間の終端アドレス
    max_virt_inclusive: usize,

    // 非標準のメモリ領域(normal, chacheable DRAM) のための配列
    inner: [TranslationDescriptor; NUM_SPECIAL_RANGES],
}

//--------------------------------------------------------------------------------------------------
// Public Code
//--------------------------------------------------------------------------------------------------

impl fmt::Display for MMUEnableError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MMUEnableError::AlreadyEnabled => write!(f, "MMU is already enabled"),
            MMUEnableError::Other(x) => write!(f, "{}", x),
        }
    }
}

impl<const GRANULE_SIZE: usize> TranslationGranule<GRANULE_SIZE> {
    //
    pub const SIZE: usize = Self::size_checked();

    pub const SHIFT: usize = Self::SIZE.trailing_zeros() as usize;

    const fn size_checked() -> usize {
        assert!(GRANULE_SIZE.is_power_of_two());

        GRANULE_SIZE
    }
}

impl<const AS_SIZE: usize> AddressSpace<AS_SIZE> {
    pub const SIZE: usize = Self::size_checked();

    pub const SIZE_SIFHT: usize = Self::SIZE.trailing_zeros() as usize;

    const fn size_checked() -> usize {
        assert!(AS_SIZE.is_power_of_two());

        Self::arch_address_space_size_sanity_checks();

        AS_SIZE
    }
}

impl Default for AttributeFields {
    fn default() -> Self {
        Self {
            mem_attributes: MemAttributes::CacheableDRAM,
            acc_perms: AcessPermissons::ReadWrite,
            execute_never: true,
        }
    }
}

// TranslationDesxriptorのためのフォーマッタ
impl fmt::Display for TranslationDescriptor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let start = *(self.virtual_range)().start();
        let end = *(self.virtual_range)().end();
        let size = end - start + 1;

        // log2(1024)
        const KIB_SHIFT: u32 = 10;

        // log2(1024 * 1024)
        const MIB_SHIFT: u32 = 10;

        let (size, unit) = if (size >> MIB_SHIFT) > 0 {
            (size >> MIB_SHIFT, "MIB")
        } else if (size >> KIB_SHIFT) > 0 {
            (size >> KIB_SHIFT, "KiB")
        } else {
            (size, "Byte")
        };

        let attr = match self.attribute_fields.mem_attributes {
            MemAttributes::CacheableDRAM => "C",
            MemAttributes::Device => "Dev",
        };

        let acc_p = match self.attribute_fields.acc_perms {
            AcessPermissons::ReadOnly => "RO",
            AcessPermissons::ReadWrite => "RW",
        };

        let xn = if self.attribute_fields.execute_never {
            "PXN"
        } else {
            "PX"
        };

        write!(
            f,
            "      {:#010x} - {:#010x} | {: >3} {} | {: <3} {} {: <3} | {}",
            start, end, size, unit, attr, acc_p, xn, self.name
        )
    }
}

impl<const NUM_SPECIAL_RANGES: usize> KernelVirtualLayout<{ NUM_SPECIAL_RANGES }> {
    pub const fn new(max: usize, layout: [TranslationDescriptor; NUM_SPECIAL_RANGES]) -> Self {
        Self {
            max_virt_inclusive: max,
            inner: layout,
        }
    }

    pub fn virt_addr_properties(
        &self,
        virt_addr: usize,
    ) -> Result<(usize, AttributeFields), &'static str> {
        if virt_addr > self.max_virt_inclusive {
            return Err("Address out of range");
        }

        for descriptor in self.inner.iter() {
            if (descriptor.virtual_range)().contains(&virt_addr) {
                let output_addr = match descriptor.physical_range_translation {
                    Translation::Identity => virt_addr,
                    Translation::Offset(offset) => {
                        offset + (virt_addr - (descriptor.virtual_range)().start())
                    }
                };
                return Ok((output_addr, descriptor.attribute_fields));
            }
        }

        Ok((virt_addr, AttributeFields::default()))
    }

    pub fn print_layout(&self) {
        use crate::info;

        for descriptor in self.inner.iter() {
            info!("{}", descriptor);
        }
    }
}
