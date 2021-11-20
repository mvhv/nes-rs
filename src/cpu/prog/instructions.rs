use crate::cpu::ops::{Mnemonic, Opcode, CPU_OPCODE_MAP};
use crate::cpu::addr::AddressMode;


#[derive(Debug)]
pub enum Operand {
    None,
    Word(u8),
    DoubleWord(u16),
}

#[derive(Debug)]
pub struct Instruction {
    opcode: &'static Opcode,
    operand: Operand,
}

impl Instruction {
    pub fn from_iter<'a, I>(program: &mut I) -> Option<Self> 
    where
        I: Iterator<Item = &'a u8>
    {
        if let Some(byte) = program.next() {
            let opcode = CPU_OPCODE_MAP.get(&byte).expect("Error, opcode not found in map while decompiling.").clone();
            
            let operand = match opcode.mode {
                AddressMode::Implicit |
                AddressMode::Accumulator => Operand::None,
                AddressMode::Immediate |
                AddressMode::ZeroPage |
                AddressMode::ZeroPageX |
                AddressMode::ZeroPageY |
                AddressMode::Relative |
                AddressMode::IndirectX |
                AddressMode::IndirectY => Operand::Word(*program.next().unwrap()),
                AddressMode::Absolute |
                AddressMode::AbsoluteX |
                AddressMode::AbsoluteY |
                AddressMode::Indirect => Operand::DoubleWord(u16::from_le_bytes([*program.next().unwrap(), *program.next().unwrap()])),
            };

            Option::Some(Instruction { opcode, operand })
        } else {
            Option::None
        }
    }
}

impl std::fmt::Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        let mnem = self.opcode.mnemonic.to_string();
        let oper = match (&self.opcode.mode, &self.operand) {
            (AddressMode::Implicit, Operand::None) => None,
            (AddressMode::Accumulator, Operand::None) => Some("A".into()), // may need to replace this
            (AddressMode::Immediate, Operand::Word(op)) => Some(format!("#{:02x}", op)),
            (AddressMode::ZeroPage, Operand::Word(op))  => Some(format!("${:02x}", op)),
            (AddressMode::ZeroPageX, Operand::Word(op))  => Some(format!("${:02x},X", op)),
            (AddressMode::ZeroPageY, Operand::Word(op))  => Some(format!("${:02x},Y", op)),
            (AddressMode::Relative, Operand::Word(op))  => Some(format!("*{:+}", op)),
            (AddressMode::Absolute, Operand::DoubleWord(op))  => Some(format!("${:04x}", op)),
            (AddressMode::AbsoluteX, Operand::DoubleWord(op)) => Some(format!("${:04x},X", op)),
            (AddressMode::AbsoluteY, Operand::DoubleWord(op)) => Some(format!("${:04x},Y", op)),
            (AddressMode::Indirect, Operand::DoubleWord(op)) => Some(format!("(${:04x})", op)),
            (AddressMode::IndirectX, Operand::Word(op))  => Some(format!("(${:02x},X)", op)),
            (AddressMode::IndirectY, Operand::Word(op))  => Some(format!("(${:02x}),Y", op)),
            _ => todo!()
        };
        if let Some(op) = oper {
            write!(f, "{} {}", mnem, op)
        } else {
            write!(f, "{}", mnem)
        }
    }
}