#[cfg(target_arch = "arrch64")]
#[path = "../_arch/aarch64/memory/mmu.rs"]
mod arch_time;

mod translation_table;

use core::{fmt, ops::RangeInclusive};

//--------------------------------------------------------------------------------------------------
// Architectural Public Reexports
//--------------------------------------------------------------------------------------------------
pub use arch_mmu::mmu;

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
