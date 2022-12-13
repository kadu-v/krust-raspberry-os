pub mod interface {
    pub use core::fmt;
    pub trait Write {
        fn write_fmt(&self, args: fmt::Arguments) -> fmt::Result;
    }

    pub trait All: Write {}
}
