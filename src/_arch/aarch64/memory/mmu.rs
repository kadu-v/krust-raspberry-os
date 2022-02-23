use crate::{
    bsp, memory,
    memory::mmu::{
        translation_table::KernelTranslationTable, TranslationGranule,
    },
};
use core::intrinsics::unlikely;
use cortex_a::{asm::barrier, registers::*};
use tock_registers::interfaces::{ReadWriteable, Readable, Writeable};

//--------------------------------------------------------------------------------------------------
// Private Definitions
//--------------------------------------------------------------------------------------------------

// Memory Management Unit type
struct MemoryManagementUnit;

//--------------------------------------------------------------------------------------------------
// Public Definitions
//--------------------------------------------------------------------------------------------------

pub type Granule512MiB = TranslationGranule<{ 512 * 1024 * 1024 }>;
pub type Granule64KiB = TranslationGranule<{ 64 * 1024 }>;

// MAIR_EL1のインデクシングのための定数
#[allow(dead_code)]
pub mod mair {
    pub const DEVICE: u64 = 0;
    pub const NORMAL: u64 = 1;
}

//--------------------------------------------------------------------------------------------------
// Global instances
//--------------------------------------------------------------------------------------------------

static mut KERNEL_TABLES: KernelTranslationTable =
    KernelTranslationTable::new();
static MMU: MemoryManagementUnit = MemoryManagementUnit;

//--------------------------------------------------------------------------------------------------
// Private Code
//--------------------------------------------------------------------------------------------------

impl<const AS_SIZE: usize> memory::mmu::AddressSpace<AS_SIZE> {
    pub const fn arch_address_space_size_sanity_checks() {
        assert!((AS_SIZE % Granule512MiB::SIZE) == 0);

        assert!((AS_SIZE <= (1 << 48)));
    }
}

impl MemoryManagementUnit {
    fn set_up_mair(&self) {
        MAIR_EL1.write(
            MAIR_EL1::Attr1_Normal_Outer::WriteBack_NonTransient_ReadWriteAlloc
                + MAIR_EL1::Attr2_Normal_Inner::WriteBack_NonTransient_ReadWriteAlloc
                + MAIR_EL1::Attr0_Device::nonGathering_nonReordering_EarlyWriteAck,
        )
    }
    fn configure_translation_control(&self) {
        let t0sz = (64 - bsp::memory::mmu::KernelAddrSpace::SIZE_SHIFT) as u64;

        TCR_EL1.write(
            TCR_EL1::TBI0::Used
                + TCR_EL1::IPS::Bits_40
                + TCR_EL1::TG0::KiB_64
                + TCR_EL1::SH0::Inner
                + TCR_EL1::ORGN0::WriteBack_ReadAlloc_WriteAlloc_Cacheable
                + TCR_EL1::IRGN0::WriteBack_ReadAlloc_WriteAlloc_Cacheable
                + TCR_EL1::EPD0::EnableTTBR0Walks
                + TCR_EL1::A1::TTBR0
                + TCR_EL1::T0SZ.val(t0sz)
                + TCR_EL1::EPD1::DisableTTBR1Walks,
        );
    }
}

//--------------------------------------------------------------------------------------------------
// Public Code
//--------------------------------------------------------------------------------------------------

// MMU インスタンスの参照を返す関数
pub fn mmu() -> &'static impl memory::mmu::interface::MMU {
    &MMU
}

//------------------------------------------------------------------------------
// OS Interface Code
//------------------------------------------------------------------------------
use memory::mmu::MMUEnableError;

impl memory::mmu::interface::MMU for MemoryManagementUnit {
    unsafe fn enable_mmu_and_caching(&self) -> Result<(), MMUEnableError> {
        // unlikey: この条件分岐は失敗する可能性が高いことをコンパイラに知らせる
        // CPUでの分岐予測で有利に働くのかもしれない
        if unlikely(self.is_enabled()) {
            return Err(MMUEnableError::AlreadyEnabled);
        }

        // 変換の粒度がサポートされていなければ、失敗させる
        if unlikely(
            !ID_AA64MMFR0_EL1.matches_all(ID_AA64MMFR0_EL1::TGran64::Supported),
        ) {
            return Err(MMUEnableError::Other(
                "Traslation granule not suported in HW",
            ));
        }

        // メモリのアトリビュートの関節参照レジスタのセットアップ
        self.set_up_mair();

        // メモリの変換テーブルのセットアップ
        KERNEL_TABLES
            .populate_tt_entries()
            .map_err(MMUEnableError::Other)?;

        // 変換テーブルのベースアドレスの設定
        TTBR0_EL1.set_baddr(KERNEL_TABLES.phys_base_address());

        self.configure_translation_control();

        // MMU の設定
        barrier::isb(barrier::SY);

        // MMUの有効化
        //
        SCTLR_EL1.modify(
            SCTLR_EL1::M::Enable
                + SCTLR_EL1::C::Cacheable
                + SCTLR_EL1::I::Cacheable,
        );

        barrier::isb(barrier::SY);

        Ok(())
    }

    #[inline(always)]
    fn is_enabled(&self) -> bool {
        SCTLR_EL1.matches_all(SCTLR_EL1::M::Enable)
    }
}
