use lazy_static::lazy_static;
use std::collections::HashMap;

use super::AddressingMode;

/// BReaK
pub const BRK: u8 = 0x00;
/// LoaD Accumulator
pub const LDA: u8 = 0xA9;
/// Transfer Accumulator to X
pub const TAX: u8 = 0xAA;
/// INcrement X
pub const INX: u8 = 0xE8;
//ComPare Y register
pub const CPY: u8 = 0xC0;

#[derive(Debug)]
pub struct Opcode {
    mnemonic: &'static str,
    code: u8,
    bytes: u8,
    cycles: u8,
    mode: AddressingMode,
}

impl Opcode {
    pub fn new(mnemonic: &'static str, code: u8, bytes: u8, cycles: u8, mode: AddressingMode) -> Self {
        Opcode { mnemonic, code, bytes, cycles, mode }
    }
}

// impl From<u8> for Opcode {
//     fn from(u8: code) -> Self {
        
//     }
// }

impl Default for Opcode {
    fn default() -> Self {
        Opcode::new(
            "NIL",
            0xFF,
            0,
            0,
            AddressingMode::None
        )
    }
}

lazy_static! {
    pub static ref CPU_OPCODES: Vec<Opcode> = vec![
        Opcode::new("BRK", 0x00, 1, 7, AddressingMode::None),
        Opcode::new("TAX", 0xaa, 1, 2, AddressingMode::None),
        Opcode::new("INX", 0xe8, 1, 2, AddressingMode::None),

        Opcode::new("LDA", 0xa9, 2, 2, AddressingMode::Immediate),
        Opcode::new("LDA", 0xa5, 2, 3, AddressingMode::ZeroPage),
        Opcode::new("LDA", 0xb5, 2, 4, AddressingMode::ZeroPageX),
        Opcode::new("LDA", 0xad, 3, 4, AddressingMode::Absolute),
        Opcode::new("LDA", 0xbd, 3, 4/*+1 if page crossed*/, AddressingMode::AbsoluteX),
        Opcode::new("LDA", 0xb9, 3, 4/*+1 if page crossed*/, AddressingMode::AbsoluteY),
        Opcode::new("LDA", 0xa1, 2, 6, AddressingMode::IndirectX),
        Opcode::new("LDA", 0xb1, 2, 5/*+1 if page crossed*/, AddressingMode::IndirectY),

        Opcode::new("STA", 0x85, 2, 3, AddressingMode::ZeroPage),
        Opcode::new("STA", 0x95, 2, 4, AddressingMode::ZeroPageX),
        Opcode::new("STA", 0x8d, 3, 4, AddressingMode::Absolute),
        Opcode::new("STA", 0x9d, 3, 5, AddressingMode::AbsoluteX),
        Opcode::new("STA", 0x99, 3, 5, AddressingMode::AbsoluteY),
        Opcode::new("STA", 0x81, 2, 6, AddressingMode::IndirectX),
        Opcode::new("STA", 0x91, 2, 6, AddressingMode::IndirectY),
    ];
    
    pub static ref CPU_OPCODES_MAP: HashMap<u8, &'static Opcode> =
        CPU_OPCODES.iter()
            .map(|op| (op.code, op))
            .collect();
}