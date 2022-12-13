//! https://github.com/RaspberryPI/firmware/wiki/Mailbox-framebuffer-interface

use super::driver::MAILBOX;
use super::mailbox::*;
use crate::synchronization::{interface::Mutex, IRQSafeNullLock};
use crate::{driver, print};
use core::fmt;
use noto_sans_mono_bitmap::{
    get_bitmap, get_bitmap_width, BitmapHeight, FontWeight,
};

//--------------------------------------------------------------------------------------------------
// Gloval Definitions
//--------------------------------------------------------------------------------------------------
pub const SCREEN_WIDTH: u32 = 1024;
pub const SCREEN_HEIGHT: u32 = 768;

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
    row: usize,
    col: usize,
}

pub struct FrameBuffer {
    inner: IRQSafeNullLock<FrameBufferInner>,
}

// RGB:
//              R         G       B
// |--------|--------|--------|--------|
//      8        8        8       8
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct RGBColor {
    pub r: u8, // 5bit
    pub g: u8, // 6bit
    pub b: u8, // 5bit
}

// pub enum Color {
//     Yellow = ,
// }

//--------------------------------------------------------------------------------------------------
// Public Code
//--------------------------------------------------------------------------------------------------

impl FrameBufferInner {
    pub const fn new() -> Self {
        Self {
            phyis_width: SCREEN_WIDTH,
            phyis_height: SCREEN_HEIGHT,
            width: 840,
            heigth: 480,
            pitch: 0,
            depth: 32,
            _x_offset: 0,
            _y_offset: 0,
            addr: 0,
            size: 0,
            row: 0,
            col: 0,
        }
    }

    unsafe fn init(&mut self) -> Result<(), &'static str> {
        // send a message via property channel 8
        let mut msg = Messege::new(8);
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

        // crate::info!("addr: {:x}, size: {:x}", self.addr, self.size);
        Ok(())
    }

    fn init_msg(&self, msg: &mut Messege) {
        // all bytes of messeage data
        msg.data[0] = 26 * 4;

        // request
        msg.data[1] = 0x0;

        // physical display settings
        msg.data[2] = 0x4_8003; // tag indetity
        msg.data[3] = 8; // value buffer size
        msg.data[4] = 8; // respronse: 1 request: 0
                         // value buffer is u8 array
        msg.data[5] = self.phyis_width;
        msg.data[6] = self.phyis_height;

        // virtual display settings
        msg.data[7] = 0x4_8004;
        msg.data[8] = 8;
        msg.data[9] = 8;
        msg.data[10] = self.width;
        msg.data[11] = self.heigth;

        // depth settings
        msg.data[12] = 0x4_8005;
        msg.data[13] = 4;
        msg.data[14] = 4;
        msg.data[15] = self.depth;

        // pitch settings
        msg.data[16] = 0x4_0008;
        msg.data[17] = 4;
        msg.data[18] = 4;
        msg.data[19] = self.pitch;

        // allocate frame buffer
        msg.data[20] = 0x4_0001;
        msg.data[21] = 8;
        msg.data[22] = 8;
        msg.data[23] = 0; // frame buffer address
        msg.data[24] = 0; // frame buffer size

        // Last buffer
        msg.data[25] = 0;
    }

    fn write_pixel(&self, y: usize, x: usize, c: RGBColor) {
        // self.depth + 7は下位４bitを繰り上げている
        let ptr = (self.addr
            + y as u32 * self.pitch
            + x as u32 * ((self.depth + 7) >> 3)) as *mut u32;
        unsafe {
            core::ptr::write_volatile(
                ptr,
                ((c.b as u32) << 16) + ((c.g as u32) << 8) + c.r as u32,
            );
        }
    }

    fn write_char(&self, y: usize, x: usize, c: char) {
        let bitmap_char =
            get_bitmap(c, FontWeight::Regular, BitmapHeight::Size16)
                .expect("unsupported char");
        for (row_i, row) in bitmap_char.bitmap().iter().enumerate() {
            for (col_i, intensity) in row.iter().enumerate() {
                let (r, g, b) =
                    (*intensity as u8, *intensity as u8, *intensity as u8);
                // let (r, g, b) = (255 - r, 255 - g, 255 - b);
                // let (r, g, b) = (255 - r, 255 - g, 255 - b);
                let rgb_32 = RGBColor { r: r, g: g, b: b };
                self.write_pixel(y + row_i, x + col_i, rgb_32);
            }
        }
    }
}

impl FrameBuffer {
    pub const fn new() -> Self {
        Self {
            inner: IRQSafeNullLock::new(FrameBufferInner::new()),
        }
    }

    pub fn write_pixel(&self, y: usize, x: usize, c: RGBColor) {
        self.inner.lock(|buff| buff.write_pixel(y, x, c))
    }

    pub fn write_char(&self, y: usize, x: usize, c: char) {
        self.inner.lock(|buff| buff.write_char(y, x, c));
    }
}

//--------------------------------------------------------------------------------------------------
// OS Interface
//--------------------------------------------------------------------------------------------------
impl driver::interface::DeviceDriver for FrameBuffer {
    fn compatible(&self) -> &'static str {
        #[cfg(feature = "bsp_rpi3")]
        return "Video Core IV";

        #[cfg(feature = "bsp_rpi4")]
        return "Video Core VI";
    }

    unsafe fn init(&self) -> Result<(), &'static str> {
        self.inner.lock(|buff| buff.init())
    }
}
