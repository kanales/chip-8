use crate::lib::n_bit;

pub const WIDTH: usize = 64;
pub const HEIGHT: usize = 32;

pub trait Screen {
    fn clear(&mut self);
    fn draw(&mut self, x: u8, y: u8, pixels: &[u8], n: u8) -> bool;
}

pub struct Buffer([u8; HEIGHT * WIDTH]);

impl Buffer {
    pub fn new() -> Self {
        Buffer([0; HEIGHT * WIDTH])
    }
}

impl Screen for Buffer {
    fn clear(&mut self) {
        for p in self.0.iter_mut() {
            *p = 0;
        }
    }

    fn draw(&mut self, x: u8, y: u8, pixels: &[u8], n: u8) -> bool {
        let mut res = false;
        let buf = &mut self.0;
        let x = x as usize;
        let y = y as usize;
        let n = n as usize;
        for r in 0..n {
            let cell = pixels[r];
            for c in 0..8 {
                let b = n_bit(cell, c as u8);

                let r = (y + r) % HEIGHT;
                let c = (x + c) % WIDTH;
                let idx = r * WIDTH + c;
                res |= buf[idx] & b != 0;
                buf[idx] ^= b;
            }
        }
        res
    }
}

impl Buffer {
    pub fn get_buffer(&self) -> &[u8] {
        &self.0
    }
}
