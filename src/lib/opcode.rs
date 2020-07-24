use crate::lib::{Address, Chip8Error, Register};

#[derive(Debug, Copy, Clone)]
pub enum Opcode {
    // CALL //
    Sys(Address), // 0NNN

    // DISPLAY //
    Cls,                          // 00E0
    Draw(Register, Register, u8), // DXYN

    // FLOW //
    Ret,           // 00EE
    Goto(Address), // 1NNN
    Jump(Address), // BNNN
    Call(Address), // 2NNN

    // COND //
    SeC(Register, u8),       // 3XNN
    SneC(Register, u8),      // 4XNN
    Se(Register, Register),  // 5XY0
    Sne(Register, Register), // 9XY0

    // CONST //
    Set(Register, u8),  // 6XNN
    AddC(Register, u8), // 7XNN

    // ASSIGN //
    Assign(Register, Register), // 8XY0

    // BITWISE //
    Or(Register, Register),  // 8XY1
    And(Register, Register), // 8XY2
    Xor(Register, Register), // 8XY3
    Shr(Register, Register), // 8XY6
    Shl(Register, Register), // 8XYE

    // MATH //
    Add(Register, Register),  // 8XY4
    Sub(Register, Register),  // 8XY4
    Subn(Register, Register), // 8XY7

    // MEM //
    SetPtr(Address),      // ANNN
    AddPtr(Register),     // FX1E
    SpriteAddr(Register), // FX29
    Dump(Register),       // FX55
    Load(Register),       // FX65

    // RAND //
    Rand(Register, u8), // CXNN

    // INPUT //
    Skip(Register),   // EX8E
    Skipn(Register),  // EX9E
    GetKey(Register), // FX0A

    // TIMER //
    GetDelay(Register), // FX07
    SetDelay(Register), // FX15

    // SOUND //
    SoundTimer(Register), // FX18

    // BDC
    SetBCD(Register), // FX33
}

use std::convert::TryFrom;

struct Digits(u8, u8, u8, u8);

impl From<u16> for Digits {
    fn from(input: u16) -> Self {
        Digits(
            ((input & 0xF000) >> 12) as u8,
            ((input & 0x0F00) >> 8) as u8,
            ((input & 0x00F0) >> 4) as u8,
            (input & 0x000F) as u8,
        )
    }
}

impl TryFrom<u16> for Opcode {
    type Error = Chip8Error;
    fn try_from(input: u16) -> Result<Opcode, Chip8Error> {
        let Digits(a, x, y, n) = input.into();
        let nnn = input & 0x0FFF;
        let nn = (y << 4) + n;
        let res = match (a, x, y, n) {
            (0x0, 0x0, 0xE, 0x0) => Opcode::Cls,
            (0x0, 0x0, 0xE, 0xE) => Opcode::Ret,
            (0x0, _, _, _) => Opcode::Sys(nnn),

            (0x1, _, _, _) => Opcode::Goto(nnn),
            (0x2, _, _, _) => Opcode::Call(nnn),
            (0x3, _, _, _) => Opcode::SeC(x, nn),
            (0x4, _, _, _) => Opcode::SneC(x, nn),
            (0x5, _, _, 0x0) => Opcode::Se(x, y),
            (0x6, _, _, _) => Opcode::Set(x, nn),
            (0x7, _, _, _) => Opcode::AddC(x, nn),

            (0x8, _, _, 0x0) => Opcode::Assign(x, y),
            (0x8, _, _, 0x1) => Opcode::Or(x, y),
            (0x8, _, _, 0x2) => Opcode::And(x, y),
            (0x8, _, _, 0x3) => Opcode::Xor(x, y),
            (0x8, _, _, 0x4) => Opcode::Add(x, y),
            (0x8, _, _, 0x5) => Opcode::Sub(x, y),
            (0x8, _, _, 0x6) => Opcode::Shr(x, y),
            (0x8, _, _, 0x7) => Opcode::Subn(x, y),
            (0x8, _, _, 0xE) => Opcode::Shl(x, y),

            (0x9, _, _, 0) => Opcode::Sne(x, y),

            (0xA, _, _, _) => Opcode::SetPtr(nnn),
            (0xB, _, _, _) => Opcode::Jump(nnn),
            (0xC, _, _, _) => Opcode::Rand(x, nn),
            (0xD, _, _, _) => Opcode::Draw(x, y, n),

            (0xE, _, 0x9, 0xE) => Opcode::Skip(x),
            (0xE, _, 0xA, 0x1) => Opcode::Skipn(x),

            (0xF, _, 0x0, 0x7) => Opcode::GetDelay(x),
            (0xF, _, 0x0, 0xA) => Opcode::GetKey(x),
            (0xF, _, 0x1, 0x5) => Opcode::SetDelay(x),
            (0xF, _, 0x1, 0x8) => Opcode::SoundTimer(x),
            (0xF, _, 0x1, 0xE) => Opcode::AddPtr(x),
            (0xF, _, 0x2, 0x9) => Opcode::SpriteAddr(x),
            (0xF, _, 0x3, 0x3) => Opcode::SetBCD(x),
            (0xF, _, 0x5, 0x5) => Opcode::Dump(x),
            (0xF, _, 0x6, 0x5) => Opcode::Load(x),
            _ => return Err(Chip8Error::UnknownOpcode(input)),
        };
        Ok(res)
    }
}
