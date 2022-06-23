pub mod interface {
    pub trait Write {
        fn draw(&self, x: usize, y: usize, c: u32);
    }

    pub trait All = Write;
}
