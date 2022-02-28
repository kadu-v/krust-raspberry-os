use core::arch::global_asm;
use cortex_a::{asm, registers::*};
use tock_registers::interfaces::Writeable;

// Assembly counterpart to this file.
global_asm!(include_str!("boot.s"));

//--------------------------------------------------------------------------------------------------
// Private Code
//--------------------------------------------------------------------------------------------------

#[inline(always)]
unsafe fn prepare_el2_to_el1_transiton(phys_boot_stack_end_exclusive: u64) {
    // Enable time counter resiters for EL1
    // AArch64 Generic Timer register summary: https://developer.arm.com/documentation/ddi0500/d/ch10s03s01

    // CNTHCTL_EL2: カウンタタイマハイパーバイザ制御レジスタ
    // 目的        : 物理カウンタからのイベントストリームの生成、EL1 から物理カウンタおよび EL1 物理タイマへのアクセスを制御

    // EL1PCEN    : EL0とEL1からのEL1物理タイマレジスタへのアクセスをEL2へトラップするかを決定する
    // 目的        : 1 -> すべての命令がトラップされない

    // EL1PCTEN   : EL0とEL1からのEL1物理カウンタレジスタへのアクセスをEL2へトラップするか決定する
    // 目的        : 1 -> すべての命令がトラップされない
    CNTHCTL_EL2.write(CNTHCTL_EL2::EL1PCEN::SET + CNTHCTL_EL2::EL1PCTEN::SET);

    // No offset for reading counters
    // CNTVOFF_EL2 : カウンタタイマ仮想オフセットレジスタ
    // 目的         : CNTPCT_EL0の物理カウントの値とCNTVCT_EL0の仮想カウントの値のオフセットを決定する
    CNTVOFF_EL2.set(0);

    // Set EL1 execution state to AArch64
    // HCR_EL2 : ハイパーバイザコンフィギュレーションレジスタ
    // 目的     : EL2への各種操作のトラップ有無を定義するなど、仮想化に関する設定制御

    // RW            : 低例外レベルへの実行状態の制御
    // EL1isAarch64  : EL1の実行状態をAarch64に設定. EL0の実行状態はEL0を実行した際のPSTATE.nRWの値で決定される
    HCR_EL2.write(HCR_EL2::RW::EL1IsAarch64);

    // Set up a simulated exception return
    //
    // First, fake a saved program status where all interuputs were masked and SP_EL1 was used as a
    // stack pointer
    SPSR_EL2.write(
        SPSR_EL2::D::Masked
            + SPSR_EL2::A::Masked
            + SPSR_EL2::I::Masked
            + SPSR_EL2::F::Masked
            + SPSR_EL2::M::EL1h,
    );

    // Second, let the link register point to kenrel_init()
    ELR_EL2.set(crate::kernel_init as *const () as u64);

    // Set up SP_EL1 (stack pointer), which will be used by EL1 once we "return" to it.
    // Since there are no plans to ever to return to EL2, just re-use the same stack.
    SP_EL1.set(phys_boot_stack_end_exclusive);
}

//-------------------------------------------------------------------------------------------------
// Public Code
//-------------------------------------------------------------------------------------------------
/// The Rust entry `kernel` binary
///
/// The function is called from thw assembley `_start` function

#[no_mangle]
pub unsafe fn _start_rust(phys_boot_stack_end_exclusive: u64) -> ! {
    prepare_el2_to_el1_transiton(phys_boot_stack_end_exclusive);

    // Use `eret` to "return" to EL1.
    // This results in execution of kernel_init() in EL1
    asm::eret();
}
