// Arm v8 アーキテクチャリファレンスマニュアル
// http://karekinada.na.coocan.jp/Jetson/Xavier.doc/ARMv8_Part_A-B/index_jp.html

use crate::{time, warn};
use core::time::Duration;
use cortex_a::{asm::barrier, registers::*};
use tock_registers::interfaces::{ReadWriteable, Readable, Writeable};

//--------------------------------------------------------------------------------------------------
// Private Definitions
//--------------------------------------------------------------------------------------------------

const NS_PER_S: u64 = 1_000_000_000;

// Arm v8 Generic Timer
// CPU内の汎用タイマー
// ARMでは組み込みでCPU内にタイマーが実装されている
struct GenericTimer;

//--------------------------------------------------------------------------------------------------
// Global instances
//--------------------------------------------------------------------------------------------------
static TIME_MANAGER: GenericTimer = GenericTimer;

//--------------------------------------------------------------------------------------------------
// Private Code
//--------------------------------------------------------------------------------------------------

impl GenericTimer {
    #[inline(always)]
    fn read_cntpct(&self) -> u64 {
        // arm プロセッサのパイプラインがフラッシュされる
        // なぜ SY構造体を引数に渡すのか?
        unsafe { barrier::isb(barrier::SY) };

        // http://karekinada.na.coocan.jp/Jetson/Xavier.doc/ARMv8_Part_D_2143-/index_jp.html
        // CNTPCT_EL0: カウンタタイマ物理カウントレジスタ
        // 目的: 64bitの物理カウント値を保持する
        CNTPCT_EL0.get()
    }
}

//--------------------------------------------------------------------------------------------------
// Private Code
//--------------------------------------------------------------------------------------------------

// Return a reference to the time manager
pub fn time_manager() -> &'static impl time::interface::TimeManager {
    &TIME_MANAGER
}

//--------------------------------------------------------------------------------------------------
// OS Interface Code
//--------------------------------------------------------------------------------------------------

impl time::interface::TimeManager for GenericTimer {
    fn resolution(&self) -> Duration {
        // CNTFRQ_EL0: カウンタタイマ周波数レジスタ
        // 目的: このレジスタはソフトウェアがシステムカウンタの周波数を検出できるように提供されています。
        //       このレジスタはシステムの初期化の際にこの値で設定される必要があります
        //       このレジスタの値はハードウェアによって解釈されることはありません
        // 1s (ns scale) / カウンタタイマの周波数 =　カウンタタイマの間隔
        Duration::from_nanos(NS_PER_S / (CNTFRQ_EL0.get()) as u64)
    }

    fn uptime(&self) -> Duration {
        let current_count: u64 = self.read_cntpct() * NS_PER_S;
        // CNTFRQ_EL0: カウンタタイマ周波数レジスタ
        // 目的: このレジスタはソフトウェアがシステムカウンタの周波数を検出できるように提供されています。
        //       このレジスタはシステムの初期化の際にこの値で設定される必要があります
        //       このレジスタの値はハードウェアによって解釈されることはありません
        // 1s (ns scale) / カウンタタイマの周波数 =　カウンタタイマの間隔
        let frq: u64 = CNTFRQ_EL0.get();

        // カウンタタイマの値 / カウンタタイマの周波数 = アップタイム
        Duration::from_nanos(current_count / frq)
    }

    fn spin_for(&self, duration: Duration) {
        // Instatly return on zero
        if duration.as_nanos() == 0 {
            return;
        }

        // Calculate the register compare value
        // CNTFRQ_EL0: カウンタタイマ周波数レジスタ
        // 目的: このレジスタはソフトウェアがシステムカウンタの周波数を検出できるように提供されています。
        //       このレジスタはシステムの初期化の際にこの値で設定される必要があります
        //       このレジスタの値はハードウェアによって解釈されることはありません
        // 1s (ns scale) / カウンタタイマの周波数 =　カウンタタイマの間隔
        let frq = CNTFRQ_EL0.get();
        let x = match frq.checked_mul(duration.as_nanos() as u64) {
            None => {
                warn!("Spin duration too long, skipping");
                return;
            }
            Some(val) => val,
        };
        let tval = x / NS_PER_S;

        // Check if it is within supproted bounds
        let warn = if tval == 0 {
            Some("smaller")
        // The upper 32 bits of CNTP_TVAL_EL0 are reserved
        } else if tval > u32::MAX as u64 {
            Some("bigger")
        } else {
            None
        };

        //
        if let Some(w) = warn {
            warn!(
                "Spin duration {} than archtecturally supported, skipping",
                w
            );
            return;
        }

        // Set the compare value register
        // CNT_TVAL_EL0: カウンタタイマ タイマ値 レジスタ
        // 目的: EL1物理タイマのタイマ値を保持する
        // このレジスタに値を書き込むとCNTP_CVAL_EL0に CNTPCT_EL0 + Timer Value(書き込んだ値)
        // が設定される
        // カウントダウンする
        CNTP_TVAL_EL0.set(tval);

        // Kick off the counting　　　　　　　　　　　　　　　　　　// Disable time interrupt
        CNTP_CTL_EL0.modify(CNTP_CTL_EL0::ENABLE::SET + CNTP_CTL_EL0::IMASK::SET);

        // ISTATUS will be '1' when cval ticks have passed
        // Busy-check it
        // タイマ条件を満たすとCNTP_CTL_EL0::ISTATUが'1'になる
        while !CNTP_CTL_EL0.matches_all(CNTP_CTL_EL0::ISTATUS::SET) {}

        // Disable counting again
        // タイマ割り込みはハード側で有効になる
        CNTP_CTL_EL0.modify(CNTP_CTL_EL0::ENABLE::CLEAR);
    }
}
