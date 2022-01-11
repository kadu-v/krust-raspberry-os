use core::cell::UnsafeCell;

pub mod interface {
    pub trait Mutex {
        type Data;
        fn lock<R>(&self, f: impl FnOnce(&mut Self::Data) -> R) -> R;
    }
}

// A pseudo-lock for teaching purposes.

pub struct NullLock<T>
where
    T: ?Sized,
{
    data: UnsafeCell<T>,
}

//-------------------------------------------------------------------------------------------------
// Public Code
//-------------------------------------------------------------------------------------------------

unsafe impl<T> Sync for NullLock<T> where T: ?Sized + Send {}
unsafe impl<T> Send for NullLock<T> where T: ?Sized + Send {}

impl<T> NullLock<T> {
    pub const fn new(data: T) -> Self {
        Self {
            data: UnsafeCell::new(data),
        }
    }
}

//-------------------------------------------------------------------------------------------------
// OS Interface Code
//-------------------------------------------------------------------------------------------------

impl<T> interface::Mutex for NullLock<T> {
    type Data = T;

    fn lock<R>(&self, f: impl FnOnce(&mut Self::Data) -> R) -> R {
        let data = unsafe { &mut *self.data.get() };
        f(data)
    }
}
