pub mod interface {
    pub use core::fmt;
    pub trait Write {
        fn write_fmt(&self, args: fmt::Arguments) -> fmt::Result;
        fn write_string(&mut self, s: &str);
    }

    pub trait All: Write {}
}
