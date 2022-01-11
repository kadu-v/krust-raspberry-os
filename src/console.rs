//-------------------------------------------------------------------------------------------------
// Public Deginitions
//-------------------------------------------------------------------------------------------------

pub mod interface {
    // Re-export
    pub use core::fmt;

    pub trait Write {
        fn write_fmt(&self, args: fmt::Arguments) -> fmt::Result;
    }

    pub trait Statistics {
        fn chars_written(&self) -> usize {
            0
        }
    }

    pub trait All = Write + Statistics;
}
