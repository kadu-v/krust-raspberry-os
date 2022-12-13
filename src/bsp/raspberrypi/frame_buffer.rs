//! https://github.com/RaspberryPI/firmware/wiki/Mailbox-framebuffer-interface

use super::driver::{FRAMEBUFFER, MAILBOX};
use super::mailbox::*;
use crate::screen;
use crate::synchronization::{interface::Mutex, IRQSafeNullLock};
use crate::{driver, info, print};
use core::fmt;
use noto_sans_mono_bitmap::{
    get_bitmap, get_bitmap_width, BitmapHeight, FontWeight,
};

//--------------------------------------------------------------------------------------------------
// Gloval Definitions
//--------------------------------------------------------------------------------------------------
pub const BUFFER_WIDTH: usize = 1024;
pub const BUFFER_HEIGHT: usize = 768;
pub const FONT_WIDTH: usize = 8;
pub const FONT_HEIGHT: usize = 16;

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
    row_position: usize,
    column_position: usize,
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
            phyis_width: BUFFER_WIDTH as u32,
            phyis_height: BUFFER_HEIGHT as u32,
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
            row_position: BUFFER_HEIGHT - FONT_HEIGHT,
            column_position: 0,
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
        self.depth = msg.data[15].read();
        self.pitch = msg.data[19].read(); // pitch
        self.addr = msg.data[23].read(); // buffer address
        self.size = msg.data[24].read(); // buffer size

        // crate::info!("addr: {:x}, size: {:x}", self.addr, self.size);
        Ok(())
    }

    fn init_msg(&self, msg: &mut Messege) {
        // all bytes of messeage data
        msg.data[0].write(26 * 4);

        // request
        msg.data[1].write(0x0);

        // physical display settings
        msg.data[2].write(0x4_8003); // tag indetity
        msg.data[3].write(8); // value buffer size
        msg.data[4].write(8); // respronse: 1 request: 0
                              // value buffer is u8 array
        msg.data[5].write(self.phyis_width);
        msg.data[6].write(self.phyis_height);

        // virtual display settings
        msg.data[7].write(0x4_8004);
        msg.data[8].write(8);
        msg.data[9].write(8);
        msg.data[10].write(self.width);
        msg.data[11].write(self.heigth);

        // depth settings
        msg.data[12].write(0x4_8005);
        msg.data[13].write(4);
        msg.data[14].write(4);
        msg.data[15].write(self.depth);

        // pitch settings
        msg.data[16].write(0x4_0008);
        msg.data[17].write(4);
        msg.data[18].write(4);
        msg.data[19].write(self.pitch);

        // allocate frame buffer
        msg.data[20].write(0x4_0001);
        msg.data[21].write(8);
        msg.data[22].write(8);
        msg.data[23].write(0); // frame buffer address
        msg.data[24].write(0); // frame buffer size

        // Last buffer
        msg.data[25].write(0);
    }

    fn read_pixel(&self, y: usize, x: usize) -> RGBColor {
        let ptr = (self.addr
            + y as u32 * self.pitch
            + x as u32 * ((self.depth + 7) >> 3)) as *mut u32;
        let ch = unsafe { core::ptr::read_volatile(ptr) };
        let r = (ch & 0b11111111_00000000_00000000) >> 16;
        let g = (ch & 0b11111111_00000000) >> 8;
        let b = ch & 0b11111111;
        RGBColor {
            r: r as u8,
            g: g as u8,
            b: b as u8,
        }
    }

    fn write_pixel(&self, y: usize, x: usize, c: RGBColor) {
        // self.depth + 7は下位４bitを繰り上げている
        let ptr = (self.addr
            + y as u32 * self.pitch
            + x as u32 * ((self.depth + 7) >> 3)) as *mut u32;
        // print!("{:?}\n", ptr);
        unsafe {
            core::ptr::write_volatile(
                ptr,
                ((c.b as u32) << 16) + ((c.g as u32) << 8) + c.r as u32,
            );
        }
    }

    fn _write_char(&self, y: usize, x: usize, c: char) {
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

    pub fn write_char(&mut self, c: char) {
        match c {
            '\n' => self.new_line(),
            _ => {
                if self.column_position >= BUFFER_WIDTH - FONT_WIDTH {
                    self.new_line();
                }

                self._write_char(self.row_position, self.column_position, c);
                self.column_position += FONT_WIDTH;
            }
        }
    }

    pub fn new_line(&mut self) {
        for row in FONT_HEIGHT..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let color = self.read_pixel(row, col);
                self.write_pixel(row - FONT_HEIGHT, col, color);
            }
        }

        for row in 0..FONT_HEIGHT {
            self.clear_row(BUFFER_HEIGHT - row);
        }
        self.column_position = 0;
    }

    pub fn clear_row(&mut self, y: usize) {
        let color = RGBColor { r: 0, g: 0, b: 0 };
        for col in 0..BUFFER_WIDTH {
            self.write_pixel(y, col, color);
        }
    }
}

impl fmt::Write for FrameBufferInner {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.chars() {
            self.write_char(c);
        }

        Ok(())
    }
}

impl FrameBuffer {
    pub const fn new() -> Self {
        Self {
            inner: IRQSafeNullLock::new(FrameBufferInner::new()),
        }
    }

    pub fn read_pixel(&self, y: usize, x: usize) -> RGBColor {
        self.inner.lock(|buff| buff.read_pixel(y, x))
    }

    pub fn write_pixel(&self, y: usize, x: usize, c: RGBColor) {
        self.inner.lock(|buff| buff.write_pixel(y, x, c))
    }

    pub fn write_char(&self, y: usize, x: usize, c: char) {
        self.inner.lock(|buff| buff.write_char(c));
    }

    pub fn clear_row(&self, y: usize) {
        self.inner.lock(|buff| buff.clear_row(y));
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

impl screen::interface::Write for FrameBuffer {
    fn write_fmt(&self, args: fmt::Arguments) -> fmt::Result {
        self.inner.lock(|buff| fmt::Write::write_fmt(buff, args))
    }
}

impl screen::interface::All for FrameBuffer {}

pub fn screen() -> &'static impl screen::interface::Write {
    &FRAMEBUFFER
}
