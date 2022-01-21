// Timer primitives

#[cfg(target_arch = "aarch64")]
#[path = "_arch/aarch64/time.rs"]
mod arch_time;

//--------------------------------------------------------------------------------------------------
// Architectural Public Reexports
//--------------------------------------------------------------------------------------------------
pub use arch_time::time_manager;

//--------------------------------------------------------------------------------------------------
// Public Definitions
//--------------------------------------------------------------------------------------------------

// Timekeeping interfaces
pub mod interface {
    use core::time::Duration;

    // Time management functions
    pub trait TimeManager {
        // タイマーの解像度
        fn resolution(&self) -> Duration;

        // デバイスの電源を入れてからの稼働時間
        // ファームウェアとブートローダーが消費する時間が含まれる
        fn uptime(&self) -> Duration;

        // 与えられた時間だけスピンする
        fn spin_for(&self, duration: Duration);
    }
}
