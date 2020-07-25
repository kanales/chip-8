use crate::lib::opcode::Opcode;
use crate::lib::screen::Buffer;
use crate::lib::screen::Screen;
use crate::lib::Chip8Error;

use std::convert::TryInto;

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

impl Machine {
    pub fn new(program: &Vec<u8>) -> Self {
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

    pub fn key_pressed(&mut self, ks: &Vec<u8>) {
        self.pressed_keys.clone_from(ks);
        println!("pk: {:?}", self.pressed_keys);
    }

    pub fn step(&mut self) -> Result<Option<&[u8]>, Chip8Error> {
        if self.pc >= 0x1000 {
            return Err(Chip8Error::EndOfMemory);
        };

        let code0 = self.memory[self.pc] as u16;
        let code1 = self.memory[self.pc + 1] as u16;
        let code: Opcode = ((code0 << 8) + code1).try_into()?;

        self.pc += 2;
        self.timer = if self.timer > 0 { self.timer - 1 } else { 0 };
        self.sound_timer = if self.timer > 0 { self.timer - 1 } else { 0 };
        if self.sound_timer > 0 {
            //println!("BEEP");
        }
        self.execute(code)
    }

    fn v_mut(&mut self, x: u8) -> &mut u8 {
        &mut self.registers[x as usize]
    }

    fn v(&self, x: u8) -> u8 {
        self.registers[x as usize]
    }

    /// Execute a single instruction
    fn execute(&mut self, instruction: Opcode) -> Result<Option<&[u8]>, Chip8Error> {
        use Opcode::*;
        match instruction {
            Sys(_) => { /* do nothing */ }
            Cls => {
                self.buffer.clear();
                return Ok(Some(self.buffer.get_buffer()));
            }
            Draw(x, y, n) => {
                *self.v_mut(0xF) = if self.buffer.draw(
                    self.v(x),
                    self.v(y),
                    &self.memory[self.i..self.i + (n as usize)],
                    n,
                ) {
                    1
                } else {
                    0
                };
                return Ok(Some(self.buffer.get_buffer()));
            }
            Ret => {
                self.pc = self.stack.pop().ok_or(Chip8Error::EmptyStack)?;
            }
            Goto(a) => {
                self.pc = a as usize;
            }
            Jump(a) => {
                self.pc = (a + self.v(0) as u16) as usize;
            }
            Call(a) => {
                self.stack.push(self.pc);
                self.pc = a as usize;
            }
            SeC(x, k) => {
                if self.v(x) == k {
                    // skip instruction
                    self.pc += 2;
                }
            }
            SneC(x, k) => {
                if self.v(x) != k {
                    // skip instruction
                    self.pc += 2;
                }
            }
            Se(x, y) => {
                if self.v(x) == self.v(y) {
                    // skip instruction
                    self.pc += 2;
                }
            }
            Sne(x, y) => {
                if self.v(x) != self.v(y) {
                    // skip instruction
                    self.pc += 2;
                }
            }
            Set(x, k) => {
                *self.v_mut(x) = k;
            }
            AddC(x, k) => {
                let res = self.v(x) as u16 + k as u16;
                *self.v_mut(0xF) = if res & 0xFF00 != 0 { 1 } else { 0 };
                *self.v_mut(x) = (res & 0xFF) as u8;
            }
            Assign(x, y) => {
                *self.v_mut(x) = self.v(y);
            }
            Or(x, y) => {
                *self.v_mut(x) |= self.v(y);
            }
            And(x, y) => {
                *self.v_mut(x) &= self.v(y);
            }
            Xor(x, y) => {
                *self.v_mut(x) ^= self.v(y);
            }
            Shr(x, _) => {
                *self.v_mut(0xF) = self.v(x) & 1;
                *self.v_mut(x) >>= 1;
            }
            Shl(x, _) => {
                *self.v_mut(0xF) = (self.v(x) & 0x80) >> 7;
                *self.v_mut(x) = (self.v(x) & 0x7F << 1) as u8;
            }
            Add(x, y) => {
                let res = self.v(x) as u16 + self.v(y) as u16;
                *self.v_mut(0xF) = if res & 0xFF00 != 0 { 1 } else { 0 };
                *self.v_mut(x) = (res & 0xFF) as u8;
            }
            Sub(x, y) => {
                let res = 0xFF00 + self.v(x) as u16 - self.v(y) as u16;
                *self.v_mut(0xF) = if res & 0xFF00 != 0xFF00 { 0 } else { 1 };
                *self.v_mut(x) = (res & 0xFF) as u8;
            }
            Subn(x, y) => {
                let res = 0xFF00 + self.v(y) as u16 - self.v(x) as u16;
                *self.v_mut(0xF) = if res & 0xFF00 != 0xFF00 { 0 } else { 1 };
                *self.v_mut(x) = (res & 0xFF) as u8;
            }
            SetPtr(a) => {
                self.i = a as usize;
            }
            AddPtr(x) => {
                self.i += self.v(x) as usize;
            }
            SpriteAddr(x) => {
                self.i = (FS_CHARLEN * (self.v(x) & 0xF)) as usize;
            }
            Dump(x) => {
                for idx in 0..=x {
                    self.memory[self.i + idx as usize] = self.v(idx);
                }
            }
            Load(x) => {
                for idx in 0..=x {
                    *self.v_mut(idx) = self.memory[self.i + idx as usize];
                }
            }
            Rand(x, k) => {
                // TODO use random gen
                *self.v_mut(x) = rand::random::<u8>() & k;
            }
            Skip(x) => {
                if self.pressed_keys.contains(&self.v(x)) {
                    self.pc += 2;
                }
            }
            Skipn(x) => {
                if !self.pressed_keys.contains(&self.v(x)) {
                    self.pc += 2;
                }
            }
            GetKey(x) => {
                if self.pressed_keys.is_empty() {
                    self.pc -= 2;
                } else {
                    *self.v_mut(x) = self.pressed_keys[0];
                }
            }
            GetDelay(x) => {
                *self.v_mut(x) = self.timer;
            }
            SetDelay(x) => {
                self.timer = self.v(x);
            }
            SoundTimer(x) => {
                self.sound_timer = self.v(x);
            }

            SetBCD(x) => {
                let (a, b, c) = to_bcd(self.v(x));
                self.memory[self.i] = a;
                self.memory[self.i + 1] = b;
                self.memory[self.i + 2] = c;
            }
        };
        Ok(None)
    }
}
