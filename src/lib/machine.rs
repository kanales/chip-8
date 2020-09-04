use crate::lib::screen::Buffer;
use crate::lib::screen::Screen;
use crate::lib::{Chip8Error, Chip8Result};

const FS_CHARLEN: u8 = 5;
const FONTSET: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

pub struct Machine {
    memory: [u8; 0x1000],
    buffer: Buffer,
    registers: [u8; 16],
    i: usize,
    stack: Vec<usize>,
    pc: usize,
    timer: u8,
    sound_timer: u8,
    pressed_keys: Vec<u8>,
}

fn to_bcd(num: u8) -> (u8, u8, u8) {
    (num / 100, (num / 10) % 10, num % 10)
}

#[test]
fn test_to_bcd() {
    let bcd = to_bcd(127);
    assert_eq!(bcd, (1, 2, 7));
}

/// Expose the value in the nth register
macro_rules! register {
    [$s:tt # $x:expr] => {
        $s.registers[$x as usize]
    };

    [$s:tt # $x:expr; $t:ty ] => {
        $s.registers[$x as usize] as $t
    };
}

pub enum Step<'a> {
    DrawAction(&'a [u8]),
    Step,
}

impl Machine {
    pub fn new(program: &[u8]) -> Self {
        let mut m = Machine {
            memory: [0; 0x1000],
            buffer: Buffer::new(),
            registers: [0; 16],
            i: 0,
            stack: Vec::new(),
            pc: 0x200,
            timer: 0,
            sound_timer: 0,
            pressed_keys: Vec::new(),
        };
        // load font set
        m.memory[..80].clone_from_slice(&FONTSET);

        // load program to memory
        m.memory[0x200..0x200 + program.len()].clone_from_slice(program);
        m
    }

    pub fn key_pressed(&mut self, ks: Vec<u8>) {
        self.pressed_keys = ks;
    }

    fn update_timer(&mut self) {
        self.timer = if self.timer > 0 { self.timer - 1 } else { 0 };
        self.sound_timer = if self.sound_timer > 0 {
            self.sound_timer - 1
        } else {
            0
        };
    }

    pub fn step(&mut self) -> Chip8Result<Step> {
        if self.pc >= 0x1000 {
            return Err(Chip8Error::EndOfMemory);
        };

        let code0 = self.memory[self.pc];
        let code1 = self.memory[self.pc + 1];

        self.update_timer();
        // instructions are 2 cells wide
        self.pc += 2;
        if self.sound_timer > 0 {
            //println!("BEEP");
        }
        self.execute(code0, code1)
    }

    fn execute(&mut self, most: u8, least: u8) -> Chip8Result<Step> {
        let a = (most & 0xF0) >> 4;
        let x = most & 0x0F;

        let y = (least & 0xF0) >> 4;
        let n = least & 0x0F;

        let code = ((most as u16) << 8) + (least as u16);
        let nnn = code & 0x0FFF;
        let nn = (y << 4) + n;

        match (a, x, y, n) {
            // clear screen
            (0x0, 0x0, 0xE, 0x0) => {
                self.buffer.clear();
                return Ok(Step::DrawAction(self.buffer.get_buffer()));
            }
            // return
            (0x0, 0x0, 0xE, 0xE) => {
                self.pc = self.stack.pop().ok_or(Chip8Error::EmptyStack)?;
            }
            // sys call
            (0x0, _, _, _) => { /* do nothing */ }

            (0x1, _, _, _) => {
                self.pc = nnn as usize;
            }
            (0x2, _, _, _) => {
                self.stack.push(self.pc);
                self.pc = nnn as usize;
            }
            (0x3, _, _, _) => {
                if register![self # x] == nn {
                    // skip instruction
                    self.pc += 2;
                }
            }
            (0x4, _, _, _) => {
                if register![self # x] != nn {
                    // skip instruction
                    self.pc += 2;
                }
            }
            (0x5, _, _, 0x0) => {
                if register![self # x] == register![self # y] {
                    // skip instruction
                    self.pc += 2;
                }
            }
            (0x6, _, _, _) => {
                register![self # x] = nn;
            }
            (0x7, _, _, _) => {
                let res = register![self # x; u16] + nn as u16;
                register![self # 0xF] = if res & 0xFF00 == 0 { 0 } else { 1 };
                register![self # x] = (res & 0xFF) as u8;
            }

            (0x8, _, _, 0x0) => {
                register![self # x] = register![self # y];
            }
            (0x8, _, _, 0x1) => {
                register![self # x] |= register![self # y];
            }
            (0x8, _, _, 0x2) => {
                register![self # x] &= register![self # y];
            }
            (0x8, _, _, 0x3) => {
                register![self # x] ^= register![self # y];
            }
            (0x8, _, _, 0x4) => {
                let res = register![self # x; u16] + register![self # y; u16];
                register![self # 0xF] = if res & 0xFF00 != 0 { 1 } else { 0 };
                register![self # x] = (res & 0xFF) as u8;
            }
            (0x8, _, _, 0x5) => {
                let res = 0xFF00 + register![self # x; u16] - register![self # y; u16];
                register![self # 0xF] = if res & 0xFF00 != 0xFF00 { 0 } else { 1 };
                register![self # x] = (res & 0xFF) as u8;
            }
            (0x8, _, _, 0x6) => {
                register![self # 0xF] = register![self # x] & 1;
                register![self # x] >>= 1;
            }
            (0x8, _, _, 0x7) => {
                let res = 0xFF00 + register![self # y; u16] - register![self # x; u16];
                register![self # 0xF] = if res & 0xFF00 != 0xFF00 { 0 } else { 1 };
                register![self # x] = (res & 0xFF) as u8;
            }
            (0x8, _, _, 0xE) => {
                register![self # 0xF] = (register![self # x] & 0x80) >> 7;
                register![self # x] = (register![self # x] & 0x7F << 1) as u8;
            }

            (0x9, _, _, 0) => {
                if register![self # x] != register![self # y] {
                    // skip instruction
                    self.pc += 2;
                }
            }

            (0xA, _, _, _) => {
                self.i = nnn as usize;
            }
            (0xB, _, _, _) => {
                self.pc = (nnn + register![self # 0; u16]) as usize;
            }
            (0xC, _, _, _) => {
                // TODO use random gen
                register![self # x] = rand::random::<u8>() & nn;
            }
            // Draw sprite
            (0xD, _, _, _) => {
                // TODO fix this
                // maybe try to notify the screen (observer style)?
                register![self # 0xF] = if self.buffer.draw(
                    register![self # x],
                    register![self # y],
                    &self.memory[self.i..self.i + (n as usize)],
                    n,
                ) {
                    1
                } else {
                    0
                };
                return Ok(Step::DrawAction(self.buffer.get_buffer()));
            }

            (0xE, _, 0x9, 0xE) => {
                if self.pressed_keys.contains(&register![self # x]) {
                    self.pc += 2;
                }
            }
            (0xE, _, 0xA, 0x1) => {
                if !self.pressed_keys.contains(&register![self # x]) {
                    self.pc += 2;
                }
            }

            (0xF, _, 0x0, 0x7) => {
                register![self # x] = self.timer;
            }
            (0xF, _, 0x0, 0xA) => {
                if self.pressed_keys.is_empty() {
                    self.pc -= 2;
                } else {
                    register![self # x] = self.pressed_keys[0];
                }
            }
            (0xF, _, 0x1, 0x5) => {
                self.timer = register![self # x];
            }
            (0xF, _, 0x1, 0x8) => {
                self.sound_timer = register![self # x];
            }
            (0xF, _, 0x1, 0xE) => {
                self.i += register![self # x] as usize;
            }
            (0xF, _, 0x2, 0x9) => {
                self.i = (FS_CHARLEN * (register![self # x] & 0xF)) as usize;
            }
            (0xF, _, 0x3, 0x3) => {
                let (a, b, c) = to_bcd(register![self # x]);
                self.memory[self.i] = a;
                self.memory[self.i + 1] = b;
                self.memory[self.i + 2] = c;
            }
            (0xF, _, 0x5, 0x5) => {
                for idx in 0..=x {
                    self.memory[self.i + idx as usize] = register![self # idx];
                }
            }
            (0xF, _, 0x6, 0x5) => {
                for idx in 0..=x {
                    register![self # idx] = self.memory[self.i + idx as usize];
                }
            }
            _ => return Err(Chip8Error::UnknownOpcode(code)),
        }
        Ok(Step::Step)
    }
}
