// Common device driver

use core::marker::PhantomData;
use core::ops;

//-------------------------------------------------------------------------------------------------
// Public Definitions
//-------------------------------------------------------------------------------------------------

pub struct MMIODerefWrapper<T> {
    pub start_addr: usize,
    phantom: PhantomData<fn() -> T>,
}

//-------------------------------------------------------------------------------------------------
// Public Code
//-------------------------------------------------------------------------------------------------

impl<T> MMIODerefWrapper<T> {
    // create a instance
    pub const unsafe fn new(start_addr: usize) -> Self {
        Self {
            start_addr,
            phantom: PhantomData,
        }
    }
}

impl<T> ops::Deref for MMIODerefWrapper<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*(self.start_addr as *const _) }
    }
}
