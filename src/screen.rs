pub mod interface {
    pub trait Write {
        fn draw(&self, x: usize, y: usize, c: usize);
    }

    pub trait All = Write;
}
