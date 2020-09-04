pub mod machine;
pub mod screen;

#[derive(Debug)]
pub enum Chip8Error {
    UnknownOpcode(u16),
    EndOfMemory,
    EmptyStack,
}

pub type Chip8Result<F> = Result<F, Chip8Error>;

pub fn n_bit(src: u8, n: u8) -> u8 {
    assert!(n < 8);
    let s = 7 - n;
    (src >> s) & 1
}

#[test]
fn test_n_bit() {
    let bits: [u8; 8] = [1, 0, 0, 1, 0, 1, 0, 0];
    let input = 0b10010100;
    for i in 0..8 {
        assert_eq!(n_bit(input, i), bits[i as usize]);
    }
}
