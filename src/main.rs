use std::collections::HashMap;

struct Address(u8, u8, u8);
type Register = u8;

struct Session;

struct Machine {
    memory: HashMap<Address, u8>,
    registers: [u8; 16],
    stack: Vec<Session>,
    pointer: Address,
}

enum Opcode {
    // CALL //
    CallMC(Address), // 0NNN

    // DISPLAY //
    Clear,                        // 00E0
    Draw(Register, Register, u8), // DXYN

    // FLOW //
    Return,        // 00EE
    Goto,          // 1NNN
    Jump(Address), // BNNN
    Call(Address), // 2NNN

    // COND //
    IfEqC(Register, u16),      // 3XNN
    IfNeC(Register, u16),      // 4XNN
    IfEq(Register, Register),  // 5XY0
    IfNeq(Register, Register), // 9XY0

    // CONST //
    Set(Register, u16),  // 6XNN
    AddC(Register, u16), // 7XNN

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
    SubI(Register, Register), // 8XY7

    // MEM //
    SetM(Address),        // ANNN
    AddM(Register),       // FX1E
    SpriteAddr(Register), // FX29
    Dump(Register),       // FX55
    Load(Register),       // FX65

    // RAND //
    Rand(Register, u16), // CXNN

    // INPUT //
    IfKey(Register),    // EX8E
    IfNotKey(Register), // EX9E
    GetKey(Register),   // FX0A

    // TIMER //
    GetDelay(Register), // FX07
    SetDelay(Register), // FX15

    // SOUND //
    SoundTimer(Register), // FX18

    // BDC
    SetBCD(Register), // FX33
}

fn main() {
    println!("Hello, world!");
}
