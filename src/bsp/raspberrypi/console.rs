use crate::console;
use crate::{synchronization, synchronization::NullLock};
use core::fmt;

//-------------------------------------------------------------------------------------------------
// Private Definitions
//-------------------------------------------------------------------------------------------------

struct QEMUOutputInner {
    chars_written: usize,
}

//-------------------------------------------------------------------------------------------------
// Public Definitions
//-------------------------------------------------------------------------------------------------
// The main struct
pub struct QEMUOutput {
    inner: NullLock<QEMUOutputInner>,
}

//-------------------------------------------------------------------------------------------------
// Public Definitions
//-------------------------------------------------------------------------------------------------

static QEMU_OUTPUT: QEMUOutput = QEMUOutput::new();

//-------------------------------------------------------------------------------------------------
// Private Code
//-------------------------------------------------------------------------------------------------
impl QEMUOutputInner {
    const fn new() -> Self {
        Self { chars_written: 0 }
    }

    fn write_char(&mut self, c: char) {
        unsafe {
            core::ptr::write_volatile(0x3f20_1000 as *mut u8, c as u8);
        }

        self.chars_written += 1;
    }
}

impl fmt::Write for QEMUOutputInner {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.chars() {
            unsafe {
                if c == '\n' {
                    self.write_char('\r');
                }
                self.write_char(c);
            }
        }
        Ok(())
    }
}

//-------------------------------------------------------------------------------------------------
// Public Code
//-------------------------------------------------------------------------------------------------
impl QEMUOutput {
    pub const fn new() -> Self {
        Self {
            inner: NullLock::new(QEMUOutputInner::new()),
        }
    }
}

pub fn console() -> &'static impl console::interface::All {
    &QEMU_OUTPUT
}

//-------------------------------------------------------------------------------------------------
// Public Code
//-------------------------------------------------------------------------------------------------
use synchronization::interface::Mutex;

impl console::interface::Write for QEMUOutput {
    fn write_fmt(&self, args: core::fmt::Arguments) -> fmt::Result {
        self.inner.lock(|inner| fmt::Write::write_fmt(inner, args))
    }
}

impl console::interface::Statistics for QEMUOutput {
    fn chars_written(&self) -> usize {
        self.inner.lock(|inner| inner.chars_written)
    }
}
