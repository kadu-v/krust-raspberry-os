use super::driver::FRAMEBUFFER;
use crate::bsp::raspberrypi::frame_buffer::{FrameBuffer, RGBColor};
use crate::bsp::raspberrypi::frame_buffer::{SCREEN_HEIGHT, SCREEN_WIDTH};
use crate::screen;
use core::fmt;

pub struct ScreenWriter {
    column_position: usize,
    // color_code: ColorCode,
    frame_buffer: &'static FrameBuffer,
}

impl ScreenWriter {
    pub fn new() -> Self {
        Self {
            column_position: 0,
            frame_buffer: &FRAMEBUFFER,
        }
    }

    pub fn write_char(&mut self, c: char) {
        match c {
            '\n' => self.new_line(),
            _ => {
                if self.column_position >= SCREEN_WIDTH as usize - 8 {
                    self.new_line();
                }

                self.frame_buffer.write_char(0, self.column_position, c);
                self.column_position += 8;
            }
        }
    }

    pub fn new_line(&mut self) {}

    pub fn write_pixel(&self, y: usize, x: usize, b: bool) {
        let c = if b {
            RGBColor {
                r: 255,
                g: 255,
                b: 255,
            }
        } else {
            RGBColor { r: 0, g: 0, b: 0 }
        };
        self.frame_buffer.write_pixel(y, x, c)
    }
}

impl screen::interface::Write for ScreenWriter {
    fn write_fmt(&self, arg: fmt::Arguments) -> fmt::Result {
        unimplemented!("write_str has not been implmented")
    }

    fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            self.write_char(byte as char);
        }
    }
}

impl screen::interface::All for ScreenWriter {}
