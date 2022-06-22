//! https://github.com/RaspberryPI/firmware/wiki/Mailbox-framebuffer-interface

use crate::synchronization::{interface::Mutex, IRQSafeNullLock};

use crate::{driver, println, screen};

use super::driver::{FRAMEBUFFER, MAILBOX};
use super::mailbox::*;

//--------------------------------------------------------------------------------------------------
// Public Definitions
//--------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, Copy)]
pub struct FrameBufferInner {
    phyis_width: u32,
    phyis_height: u32,
    width: u32,
    heigth: u32,
    pitch: u32,
    depth: u32,
    _x_offset: u32,
    _y_offset: u32,
    addr: u32,
    size: u32,
}

pub struct FrameBuffer {
    inner: IRQSafeNullLock<FrameBufferInner>,
}

// #[derive(Debug, Clone, Copy)]
// pub struct Rgb {
//     r: u32,
//     g: u32,
//     b: u32,
// }

// pub enum Color {
//     Yellow = ,
// }

//--------------------------------------------------------------------------------------------------
// Public Code
//--------------------------------------------------------------------------------------------------

impl FrameBufferInner {
    pub const fn new() -> Self {
        Self {
            phyis_width: 1024,
            phyis_height: 768,
            width: 640,
            heigth: 480,
            pitch: 0,
            depth: 16,
            _x_offset: 0,
            _y_offset: 0,
            addr: 0,
            size: 0,
        }
    }

    unsafe fn init(&mut self) -> Result<(), &'static str> {
        // send a message via property channel 8
        let mut msg = Messeage::new(8);

        // init message for frame buffer
        self.init_msg(&mut msg);

        // send a messeage
        if let Err(_e) = MAILBOX.mailbox_call(&mut msg) {
            return Err("MailBox Error");
        }

        // set a settings
        self.depth = msg.data[15];
        self.pitch = msg.data[19]; // pitch
        self.addr = msg.data[23]; // buffer address
        self.size = msg.data[24]; // buffer size

        // panic!("addr: {:x}, size: {:x}", self.addr, self.size);
        Ok(())
    }

    fn init_msg(&self, msg: &mut Messeage) {
        // all bytes of messeage data
        msg.data[0] = 112;

        // request
        msg.data[1] = 0x0;

        // physical display settings
        msg.data[2] = 0x4_8003; // tag indetity
        msg.data[3] = 8; // value buffer size
        msg.data[4] = 0; // respronse: 1 request: 0
                         // value buffer is u8 array
        msg.data[5] = self.phyis_width;
        msg.data[6] = self.phyis_height;

        // virtual display settings
        msg.data[7] = 0x4_8004;
        msg.data[8] = 8;
        msg.data[9] = 0;
        msg.data[10] = self.width;
        msg.data[11] = self.heigth;

        // depth settings
        msg.data[12] = 0x4_8005;
        msg.data[13] = 4;
        msg.data[14] = 0;
        msg.data[15] = self.depth;

        // pitch settings
        msg.data[16] = 0x4_0008;
        msg.data[17] = 4;
        msg.data[18] = 0;
        msg.data[19] = self.pitch;

        // allocate frame buffer
        msg.data[20] = 0x4_0001;
        msg.data[21] = 8;
        msg.data[22] = 0;
        msg.data[23] = 0; // frame buffer address
        msg.data[24] = 0; // frame buffer size

        // Last buffer
        msg.data[25] = 0;
    }

    pub fn draw(&self, x: usize, y: usize, c: u16) {
        // self.depth + 7は下位４bitを繰り上げている
        let ptr = (self.addr
            + y as u32 * self.pitch
            + x as u32 * ((self.depth + 7) >> 3)) as *mut u16;
        unsafe {
            core::ptr::write_volatile(ptr, c);
        }
    }
}

impl FrameBuffer {
    pub const fn new() -> Self {
        Self {
            inner: IRQSafeNullLock::new(FrameBufferInner::new()),
        }
    }

    pub fn draw(&self, x: usize, y: usize, c: usize) {
        self.inner.lock(|buff| buff.draw(x, y, c as u16))
    }
}

//--------------------------------------------------------------------------------------------------
// OS Interface
//--------------------------------------------------------------------------------------------------

impl driver::interface::DeviceDriver for FrameBuffer {
    fn compatible(&self) -> &'static str {
        "Video Core VI"
    }

    unsafe fn init(&self) -> Result<(), &'static str> {
        self.inner.lock(|buff| buff.init())
    }
}

impl screen::interface::Write for FrameBuffer {
    fn draw(&self, x: usize, y: usize, c: usize) {
        self.draw(x, y, c);
    }
}

pub fn screen() -> &'static impl screen::interface::All {
    &FRAMEBUFFER
}
