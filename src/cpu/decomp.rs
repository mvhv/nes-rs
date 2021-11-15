// use nom::{IResult, digit, types::CompleteStr};

use std::error::Error;

use crate::cpu::ops::{Mnemonic, Opcode, CPU_OPCODES_MAP};
use crate::cpu::addr::AddressMode;

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
            let opcode = CPU_OPCODES_MAP.get(&byte).expect("Error, opcode not found in map while decompiling.").clone();
            
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
        let mnem = self.opcode.mnemonic.as_string();
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

#[derive(Debug)]
pub enum Operand {
    None,
    Word(u8),
    DoubleWord(u16),
}

pub fn decompile_to_string(program: &[u8]) -> Result<String, Box<dyn Error>> {
    Ok(
        decompile_to_instructions(program)?
            .iter()
            .map(|inst| format!("{}", inst))
            .collect::<Vec<String>>()
            .join("\n")
    )
}

pub fn decompile_to_instructions(program: &[u8]) -> Result<Vec<Instruction>, Box<dyn Error>> {
    let mut instructions = Vec::new();
    let mut cursor = program.iter();

    while let Some(instruction) = Instruction::from_iter(&mut cursor) {
        instructions.push(instruction);
    }

    Ok(instructions)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_print_decompilation() {
        // LDA #23
        // BRK
        let program = &[0xA9, 0x23, 0x00];
        let decomp = decompile_to_string(program).expect("Failed to unwrap in test_print_decompilation.");
        assert_eq!(decomp, "LDA #23\nBRK");
    }
}