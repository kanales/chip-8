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
        for row in 0..n {
            let cell = pixels[row];
            for col in 0..8 {
                let value = n_bit(cell, col as u8);
                let row = (y + row) % HEIGHT;
                let col = (x + col) % WIDTH;
                let idx = row * WIDTH + col;
                res |= buf[idx] & value != 0;
                buf[idx] ^= value;
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
