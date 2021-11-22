mod instructions;
mod parse;

use std::{fmt::Display, str::FromStr};

use nom::IResult;
use nom::{bytes::complete::take, };

use instructions::Instruction;
use crate::cpu::ops::Mnemonic;

pub struct Program {
    start: u16,
    code: Vec<Instruction>
}

impl Program {
    /// Initialises an empty program with a start index of 0
    pub fn new() -> Self {
        Program { start: 0, code: Vec::new() }
    }
}

impl FromStr for Program {
    type Err = Box<dyn std::error::Error>;

    fn from_str(_s: &str) -> Result<Self, Self::Err> {
        Ok(Program::new())
    }
}

impl TryFrom<&[u8]> for Program {
    type Error = Box<dyn std::error::Error>;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let mut instructions = Vec::new();
        
        let mut cursor = value.iter();
        while let Some(instruction) = Instruction::from_iter(&mut cursor) {
            instructions.push(instruction);
        }

        Ok(Program { start: 0, code: instructions })
    }
}

impl Display for Program {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        for instruction in &self.code {
            write!(f, "{}\n", instruction)?;
        }

        Ok(())
    }
}

// pub fn assemble(program: &str) -> Result<Program, Box<dyn std::error::Error>> {
//     Ok(Program::new())
// }

// pub fn decompile_to_string(program: &[u8]) -> Result<String, Box<dyn Error>> {
//     Ok(
//         decompile_to_instructions(program)?
//             .iter()
//             .map(|inst| format!("{}", inst))
//             .collect::<Vec<String>>()
//             .join("\n")
//     )
// }

pub fn disassemble(code: &[u8]) -> Result<Program, Box<dyn std::error::Error>> {
    Program::try_from(code)
}

// pub fn assemble(program: &str) -> Result<&[u8], Box<dyn std::error::Error>> {
//     Program::try_from(code)
// }


/// Implementation of Snake for MOS 6502 with memory-mapped display
/// Annotated source avaliable here: https://gist.github.com/wkjagt/9043907
pub const SNAKE_BYTES: &[u8] = &[
    0x20, 0x06, 0x06, 0x20, 0x38, 0x06, 0x20, 0x0d, 0x06, 0x20, 0x2a, 0x06, 0x60, 0xa9, 0x02, 0x85,
    0x02, 0xa9, 0x04, 0x85, 0x03, 0xa9, 0x11, 0x85, 0x10, 0xa9, 0x10, 0x85, 0x12, 0xa9, 0x0f, 0x85,
    0x14, 0xa9, 0x04, 0x85, 0x11, 0x85, 0x13, 0x85, 0x15, 0x60, 0xa5, 0xfe, 0x85, 0x00, 0xa5, 0xfe,
    0x29, 0x03, 0x18, 0x69, 0x02, 0x85, 0x01, 0x60, 0x20, 0x4d, 0x06, 0x20, 0x8d, 0x06, 0x20, 0xc3,
    0x06, 0x20, 0x19, 0x07, 0x20, 0x20, 0x07, 0x20, 0x2d, 0x07, 0x4c, 0x38, 0x06, 0xa5, 0xff, 0xc9,
    0x77, 0xf0, 0x0d, 0xc9, 0x64, 0xf0, 0x14, 0xc9, 0x73, 0xf0, 0x1b, 0xc9, 0x61, 0xf0, 0x22, 0x60,
    0xa9, 0x04, 0x24, 0x02, 0xd0, 0x26, 0xa9, 0x01, 0x85, 0x02, 0x60, 0xa9, 0x08, 0x24, 0x02, 0xd0,
    0x1b, 0xa9, 0x02, 0x85, 0x02, 0x60, 0xa9, 0x01, 0x24, 0x02, 0xd0, 0x10, 0xa9, 0x04, 0x85, 0x02,
    0x60, 0xa9, 0x02, 0x24, 0x02, 0xd0, 0x05, 0xa9, 0x08, 0x85, 0x02, 0x60, 0x60, 0x20, 0x94, 0x06,
    0x20, 0xa8, 0x06, 0x60, 0xa5, 0x00, 0xc5, 0x10, 0xd0, 0x0d, 0xa5, 0x01, 0xc5, 0x11, 0xd0, 0x07,
    0xe6, 0x03, 0xe6, 0x03, 0x20, 0x2a, 0x06, 0x60, 0xa2, 0x02, 0xb5, 0x10, 0xc5, 0x10, 0xd0, 0x06,
    0xb5, 0x11, 0xc5, 0x11, 0xf0, 0x09, 0xe8, 0xe8, 0xe4, 0x03, 0xf0, 0x06, 0x4c, 0xaa, 0x06, 0x4c,
    0x35, 0x07, 0x60, 0xa6, 0x03, 0xca, 0x8a, 0xb5, 0x10, 0x95, 0x12, 0xca, 0x10, 0xf9, 0xa5, 0x02,
    0x4a, 0xb0, 0x09, 0x4a, 0xb0, 0x19, 0x4a, 0xb0, 0x1f, 0x4a, 0xb0, 0x2f, 0xa5, 0x10, 0x38, 0xe9,
    0x20, 0x85, 0x10, 0x90, 0x01, 0x60, 0xc6, 0x11, 0xa9, 0x01, 0xc5, 0x11, 0xf0, 0x28, 0x60, 0xe6,
    0x10, 0xa9, 0x1f, 0x24, 0x10, 0xf0, 0x1f, 0x60, 0xa5, 0x10, 0x18, 0x69, 0x20, 0x85, 0x10, 0xb0,
    0x01, 0x60, 0xe6, 0x11, 0xa9, 0x06, 0xc5, 0x11, 0xf0, 0x0c, 0x60, 0xc6, 0x10, 0xa5, 0x10, 0x29,
    0x1f, 0xc9, 0x1f, 0xf0, 0x01, 0x60, 0x4c, 0x35, 0x07, 0xa0, 0x00, 0xa5, 0xfe, 0x91, 0x00, 0x60,
    0xa6, 0x03, 0xa9, 0x00, 0x81, 0x10, 0xa2, 0x00, 0xa9, 0x01, 0x81, 0x10, 0x60, 0xa2, 0x00, 0xea,
    0xea, 0xca, 0xd0, 0xfb, 0x60,
];


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_disassemble_bytes() {
        // LDA #23
        // BRK
        let program = Program::try_from(&[0xA9, 0x23, 0x00][..]).unwrap();
        assert_eq!(format!("{}", program), "LDA #23\nBRK\n");
    }

    #[test]
    fn test_disassemble_snake() {
        // disassemble the snake program just check for exceptions
        // may later try to compare it against the source
        let _: Program = SNAKE_BYTES.try_into().unwrap();
    }
}