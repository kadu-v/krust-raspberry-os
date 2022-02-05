use crate::{
    bsp, memory,
    memory::mmu::{
        arch_mmu::{Granule512MiB, Granule64KiB},
        AccessPermissions, AttributeFields, MemAttributes,
    },
};

use core::convert;
use tock_registers::{
    interfaces::{Readable, Writeable},
    register_bitfields,
    registers::InMemoryRegister,
};

use self::STAGE1_PAGE_DESCRIPTOR::TYPE::Page;

//--------------------------------------------------------------------------------------------------
// Private Definitions
//--------------------------------------------------------------------------------------------------

// level 2 translation table       table descriptor              page descriptor
// TTBR_EL1 レジスタ           -->  level 2 translation table --> level 3 translation table

// table descriptor
//  64                47                            16          1 0
// |-----------------|--------------------------------|--------|-|-|
// |    ignore       |              32                |        |1|1|
// |-----------------|--------------------------------|--------|-|-|
//                    つぎのレベルの変換テーブルのアドレス

// ディスクリプタテーブル,
// ARMv8-A アーキテクチャリファレンスマニュアル　Figure D5-15
register_bitfields! {u64,
    STAGE1_TABLE_DESCRIPTOR [
        // つぎの変換テーブルの物理アドレス
        NEXT_LEVEL_TABLE_ADDR_64KiB OFFSET(16) NUMBITS(32) [],

        TYPE OFFSET(1) NUMBITS(1) [
            Block = 0,
            Table = 1
        ],

        VALID OFFSET(0) NUMBITS(1) [
            False = 0,
            True = 1,
        ]
    ]
}

// page descriptor
//  64        54 55      47                            16    10 8  6        1 0
// |----------|-|-|-----|--------------------------------|--|--|--|--|-----|-|-|
// |          | | |     |              32                |  |2 |2 |2 |     |1|1|
// |----------|-|-|-----|--------------------------------|--|--|--|--|-----|-|-|
//                        つぎのlevel2の変換テーブル(table descriptor)の物理アドレス
//                        またはlevel3の変換テーブル(page descriptor)の物理アドレス

// 物理アドレス空間の一つのブロックがFrame
// 仮想メモリ空間の一つのブロックがPage

// レベル3ページディスクリプタ
register_bitfields! {u64,
    STAGE1_PAGE_DESCRIPTOR [
        // 特権を持っていないものが実行できる
        UXN OFFSET(54) NUMBITS(1) [
            False = 0,
            True = 1
        ],

        // 特権をもっていれば実行できる
        PXN OFFSET(53) NUMBITS(1) [
            False = 0,
            True = 1
        ],

        // つぎのlevel2の変換テーブル(table descriptor)の物理アドレス
        // またはlevel3の変換テーブル(page descriptor)の物理アドレス
        OUTPUT_ADDR_64KiB OFFSET(16) NUMBITS(32) [],

        // Access Flag
        AF OFFSET(10) NUMBITS(1) [
            False = 0,
            True = 1
        ],

        //
        SH OFFSET(8) NUMBITS(2) [
            OuterShareable = 0b10,
            InnerShareable = 0b11
        ],

        AP OFFSET(6) NUMBITS(2) [
            RW_EL1 = 0b00,
            RW_EL1_EL0 = 0b01,
            R0_EL1 = 0b10,
            R0_EL1_EL0 = 0b11
        ],

        AttrIndx OFFSET(2) NUMBITS(3) [],

        TYPE OFFSET(1) NUMBITS(1) [
            Reserved_Invalid = 0,
            Page = 1
        ],

        VALID OFFSET(0) NUMBITS(1) [
            False = 0,
            True = 1
        ]
    ]
}

#[derive(Clone, Copy)]
#[repr(C)]
struct TableDescriptor {
    value: u64,
}

#[derive(Clone, Copy)]
#[repr(C)]
struct PageDescriptor {
    value: u64,
}

trait StartAddr {
    fn phys_start_addr_u64(&self) -> u64;
    fn phys_start_addr_usize(&self) -> usize;
}

const NUM_LVL2_TABLES: usize = bsp::memory::mmu::KernelAddrSpace::SIZE >> Granule512MiB::SHIFT;

//--------------------------------------------------------------------------------------------------
// Public Definitions
//--------------------------------------------------------------------------------------------------

// 変換テーブルのためのモノリシック構造体
// 各レベルの変換テーブルは64KiBにアラインされなければならないので、level3 が最初に配置される
#[repr(C)]
#[repr(align(65536))]
pub struct FixedSizeTranslationTable<const NUM_TABLES: usize> {
    // ページディスクリプタ
    // 各エントリで64KiBのウィンドウを網羅
    lvl3: [[PageDescriptor; 8192]; NUM_TABLES],

    // テーブルディスクリプタ
    // 512MiBのウィンドウをもらう
    lvl2: [TableDescriptor; NUM_TABLES],
}

// カーネル空間での変換テーブルの型
pub type KernelTranslationTable = FixedSizeTranslationTable<NUM_LVL2_TABLES>;

//--------------------------------------------------------------------------------------------------
// Private Code
//--------------------------------------------------------------------------------------------------

// バイナリはまだ恒等変換なので、変換する必要はない
impl<T, const N: usize> StartAddr for [T; N] {
    fn phys_start_addr_u64(&self) -> u64 {
        self as *const _ as u64
    }

    fn phys_start_addr_usize(&self) -> usize {
        self as *const _ as usize
    }
}

impl TableDescriptor {
    // ディスクリプタはインバリッドがデフォルト
    pub const fn new_zeroed() -> Self {
        Self { value: 0 }
    }

    // 次のレベルのテーブルディスクリプタを作成する
    pub fn from_next_lvl_table_addr(phys_next_lvl_table_addr: usize) -> Self {
        let val = InMemoryRegister::<u64, STAGE1_TABLE_DESCRIPTOR::Register>::new(0);

        let shifted = phys_next_lvl_table_addr >> Granule64KiB::SHIFT;
        val.write(
            STAGE1_TABLE_DESCRIPTOR::NEXT_LEVEL_TABLE_ADDR_64KiB.val(shifted as u64)
                + STAGE1_TABLE_DESCRIPTOR::TYPE::Table
                + STAGE1_TABLE_DESCRIPTOR::VALID::True,
        );

        TableDescriptor { value: val.get() }
    }
}

// カーネルのメモリアトリビュートをハードウェアのMMUのアトリビュートに変換する
impl convert::From<AttributeFields>
    for tock_registers::fields::FieldValue<u64, STAGE1_PAGE_DESCRIPTOR::Register>
{
    fn from(attribute_fields: AttributeFields) -> Self {
        let mut desc = match attribute_fields.mem_attributes {
            MemAttributes::CacheableDRAM => {
                STAGE1_PAGE_DESCRIPTOR::SH::InnerShareable
                    + STAGE1_PAGE_DESCRIPTOR::AttrIndx.val(memory::mmu::arch_mmu::mair::NORMAL)
            }
            MemAttributes::Device => {
                STAGE1_PAGE_DESCRIPTOR::SH::OuterShareable
                    + STAGE1_PAGE_DESCRIPTOR::AttrIndx.val(memory::mmu::arch_mmu::mair::DEVICE)
            }
        };

        // アクセスパーミッション
        desc += match attribute_fields.acc_perms {
            AccessPermissions::ReadOnly => STAGE1_PAGE_DESCRIPTOR::AP::R0_EL1,
            AccessPermissions::ReadWrite => STAGE1_PAGE_DESCRIPTOR::AP::RW_EL1,
        };

        // executer-never アトリビュートはPXNにマップされる
        desc += if attribute_fields.execute_never {
            STAGE1_PAGE_DESCRIPTOR::PXN::True
        } else {
            STAGE1_PAGE_DESCRIPTOR::PXN::False
        };

        //
        desc += STAGE1_PAGE_DESCRIPTOR::UXN::True;

        desc
    }
}

impl PageDescriptor {
    // コンストラクタ
    pub const fn new_zeroed() -> Self {
        Self { value: 0 }
    }

    pub fn from_output_addr(phys_output_addr: usize, attribute_fields: &AttributeFields) -> Self {
        let val = InMemoryRegister::<u64, STAGE1_PAGE_DESCRIPTOR::Register>::new(0);

        let shifted = phys_output_addr as u64 >> Granule64KiB::SHIFT;
        val.write(
            STAGE1_PAGE_DESCRIPTOR::OUTPUT_ADDR_64KiB.val(shifted)
                + STAGE1_PAGE_DESCRIPTOR::AF::True
                + STAGE1_PAGE_DESCRIPTOR::TYPE::Page
                + STAGE1_PAGE_DESCRIPTOR::VALID::True
                + (*attribute_fields).into(),
        );

        Self { value: val.get() }
    }
}

//--------------------------------------------------------------------------------------------------
// Public Code
//--------------------------------------------------------------------------------------------------
impl<const NUM_TABLES: usize> FixedSizeTranslationTable<NUM_TABLES> {
    pub const fn new() -> Self {
        assert!(NUM_TABLES > 0);

        Self {
            lvl3: [[PageDescriptor::new_zeroed(); 8192]; NUM_TABLES],
            lvl2: [TableDescriptor::new_zeroed(); NUM_TABLES],
        }
    }

    pub unsafe fn populate_tt_entries(&mut self) -> Result<(), &'static str> {
        for (l2_nr, l2_entry) in self.lvl2.iter_mut().enumerate() {
            *l2_entry =
                TableDescriptor::from_next_lvl_table_addr(self.lvl3[l2_nr].phys_start_addr_usize());

            for (l3_nr, l3_entry) in self.lvl3[l2_nr].iter_mut().enumerate() {
                let virt_addr = (l2_nr << Granule512MiB::SHIFT) + (l3_nr << Granule64KiB::SHIFT);

                let (phys_output_addr, attribute_fields) =
                    bsp::memory::mmu::virt_mem_layout().virt_addr_properties(virt_addr)?;

                *l3_entry = PageDescriptor::from_output_addr(phys_output_addr, &attribute_fields);
            }
        }
        Ok(())
    }

    pub fn phys_base_address(&self) -> u64 {
        self.lvl2.phys_start_addr_u64()
    }
}
