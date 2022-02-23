//! https://github.com/RaspberryPI/firmware/wiki/Mailbox-framebuffer-interface

use crate::{driver, println, screen};

use super::{
    mailbox::{self, *},
    MAILBOX,
};

//--------------------------------------------------------------------------------------------------
// Public Definitions
//--------------------------------------------------------------------------------------------------

#[repr(C, align(16))]
pub struct FrameBuffer {
    phyis_width: u32,
    phyis_height: u32,
    width: u32,
    hegith: u32,
    pitch: u32,
    depth: u32,
    x_offset: u32,
    y_offset: u32,
    addr: u32,
    size: u32,
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

impl FrameBuffer {
    pub const fn new() -> Self {
        Self {
            phyis_width: 1920,
            phyis_height: 1080,
            width: 480,
            hegith: 270,
            pitch: 0,
            depth: 16,
            x_offset: 0,
            y_offset: 0,
            addr: 0,
            size: 0,
        }
    }

    pub fn draw(&self, x: usize, y: usize, c: u16) {
        let ptr =
            (self.addr + y as u32 * self.pitch + x as u32 * 3) as *mut u16;
        unsafe {
            *ptr = c;
        }
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
        let buff_ptr = self as *const _ as u32;
        let msg = Messeage::new(buff_ptr, 1);
        println!("Messeage: {{ data: {:x}, ch: {:x}}}", msg.data, msg.channel);

        MAILBOX
            .write_mailbox(&msg)
            .map_err(|_| "can not write mailbox for frame buffer")?;
        MAILBOX
            .read_mailbox(1)
            .map_err(|_| "can not read mailbox for frame buffer")?;

        if self.addr == 0 {
            return Err("can not get a valid address of frame buffer");
        }

        Ok(())
    }
}

impl screen::Interface::Write for FrameBuffer {
    fn draw(&self, x: usize, y: usize, c: usize) {
        self.draw(x, y, c as u16)
    }
}

pub fn screen() -> &'static impl screen::Interface::All {
    &super::FRAMEBUFFER
}
