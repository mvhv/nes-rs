use lazy_static::lazy_static;
use std::collections::HashMap;

use crate::cpu::addr::AddressMode::{self, *};

#[derive(Debug)]
pub enum Mnemonic {
    /// Add with carry
    ADC,
    /// Bitwise AND with accumulator
    AND,
    /// Arithmetic shift left
    ASL,
    /// Test bits
    BIT,
    /// Branch instructions
    BPL,
    BMI,
    BVC,
    BVS,
    BCC,
    BCS,
    BNE,
    BEQ,
    /// Break
    BRK,
    /// Compare accumulator
    CMP,
    /// Compare X register
    CPX,
    /// Compare Y register
    CPY,
    /// Decrement memory
    DEC,
    /// Bitwise exclusive OR
    EOR,
    /// Flag (processor status) instructions
    CLC,
    SEC,
    CLI,
    SEI,
    CLV,
    CLD,
    SED,
    /// Increment memory
    INC,
    /// Jump
    JMP,
    /// Jump to subroutine
    JSR,
    /// Load accumulator
    LDA,
    /// Load X register
    LDX,
    /// Load Y register
    LDY,
    /// Logical shift right
    LSR,
    /// No operation
    NOP,
    /// Bitwise OR with accumulator
    ORA,
    /// Register instructions
    TAX,
    TXA,
    DEX,
    INX,
    TAY,
    TYA,
    DEY,
    INY,
    /// Rotate left
    ROL,
    /// Rotate right
    ROR,
    /// Return from interrupt
    RTI,
    /// Return from subroutine
    RTS,
    /// Subtract with carry
    SBC,
    /// Store accumulator
    STA,
    /// Stack instructions
    TXS,
    TSX,
    PHA,
    PLA,
    PHP,
    PLP,
    /// Store X register
    STX,
    /// Store Y register
    STY,    
}

#[derive(Debug)]
pub struct Opcode {
    pub mnemonic: Mnemonic,
    pub code: u8,
    pub bytes: u8,
    pub cycles: u8,
    pub page_fault_penalty: u8,
    pub mode: AddressMode,
}

impl Opcode {
    pub fn new(mnemonic: Mnemonic, code: u8, bytes: u8, cycles: u8, page_fault_penalty: u8, mode: AddressMode) -> Self {
        Opcode { mnemonic, code, bytes, cycles, page_fault_penalty, mode }
    }
}

// impl From<u8> for Opcode {
//     fn from(u8: code) -> Self {
        
//     }
// }

// impl Default for Opcode {
//     fn default() -> Self {
//         Opcode::new(
//             "KIL",
//             0xFF,
//             0,
//             0,
//             0,
//             AddressMode::NoAddressing,
//         )
//     }
// }
use Mnemonic::*;

lazy_static! {
    /// NMOS 6502 lookup table, generated with data from
    /// http://www.6502.org/tutorials/6502opcodes.html
    pub static ref NMOS_6502_OPCODES: Vec<Opcode> = vec![
        // ADC - add with carry
        // results are dependant on the decimal flag
        // N V Z C
        Opcode::new(ADC, 0x69, 2, 2, 0, Immediate),
        Opcode::new(ADC, 0x65, 2, 3, 0, ZeroPage),
        Opcode::new(ADC, 0x75, 2, 4, 0, ZeroPageX),
        Opcode::new(ADC, 0x6D, 3, 4, 0, Absolute),
        Opcode::new(ADC, 0x7D, 3, 4, 1, AbsoluteX),
        Opcode::new(ADC, 0x79, 3, 4, 1, AbsoluteY),
        Opcode::new(ADC, 0x61, 2, 6, 0, ZeroPageX),
        Opcode::new(ADC, 0x71, 2, 5, 1, ZeroPageX),

        // AND - bitwise and with accumulator
        // N Z
        Opcode::new(AND, 0x29, 2, 2, 0, Immediate),
        Opcode::new(AND, 0x25, 2, 3, 0, ZeroPage),
        Opcode::new(AND, 0x35, 2, 4, 0, ZeroPageX),
        Opcode::new(AND, 0x2D, 3, 4, 0, Absolute),
        Opcode::new(AND, 0x3D, 3, 4, 1, AbsoluteX),
        Opcode::new(AND, 0x39, 3, 4, 1, AbsoluteY),
        Opcode::new(AND, 0x21, 2, 6, 0, IndirectX),
        Opcode::new(AND, 0x31, 2, 5, 1, IndirectX),
        
        // ASL - arithmetic shift left
        // 0 shifted into bit-0 and bit-7 is shifted into carry
        // N Z C
        Opcode::new(ASL, 0x0A, 1, 2, 0, Accumulator),
        Opcode::new(ASL, 0x06, 2, 5, 0, ZeroPage),
        Opcode::new(ASL, 0x16, 2, 6, 0, ZeroPageX),
        Opcode::new(ASL, 0x0E, 3, 6, 0, Absolute),
        Opcode::new(ASL, 0x1E, 3, 7, 0, AbsoluteX),
        
        // BIT - test bits
        // N V Z
        Opcode::new(BIT, 0x24, 2, 3, 0, ZeroPage),
        Opcode::new(BIT, 0x2C, 3, 4, 0, Absolute),

        // BXX - branch instructions
        // Branch is 2 cycles, +1 if taken, +1 if page fault
        // No flags
        Opcode::new(BPL, 0x10, 2, 2, 1, Relative), // branch on plus
        Opcode::new(BMI, 0x30, 2, 2, 1, Relative), // branch on minus
        Opcode::new(BVC, 0x50, 2, 2, 1, Relative), // branch on overflow clear
        Opcode::new(BVS, 0x70, 2, 2, 1, Relative), // branch on overflow set
        Opcode::new(BCC, 0x90, 2, 2, 1, Relative), // branch on carry clear
        Opcode::new(BCS, 0xB0, 2, 2, 1, Relative), // branch on carry set
        Opcode::new(BNE, 0xD0, 2, 2, 1, Relative), // branch on not equal
        Opcode::new(BEQ, 0xF0, 2, 2, 1, Relative), // branch on equal

        // BRK - break
        // Triggers a non-maskable interrupt and increments the PC
        // B
        Opcode::new(BRK, 0x00, 1, 7, 0, Implicit),

        // CMP - compare accumulator
        // Sets flags as if a subtraction was carried out
        // N Z C
        Opcode::new(CMP, 0xC9, 2, 2, 0, Immediate),
        Opcode::new(CMP, 0xC5, 2, 3, 0, ZeroPage),
        Opcode::new(CMP, 0xD5, 2, 4, 0, ZeroPageX),
        Opcode::new(CMP, 0xCD, 3, 4, 0, Absolute),
        Opcode::new(CMP, 0xDD, 3, 4, 1, AbsoluteX),
        Opcode::new(CMP, 0xD9, 3, 4, 1, AbsoluteY),
        Opcode::new(CMP, 0xC1, 2, 6, 0, IndirectX),
        Opcode::new(CMP, 0xD1, 2, 5, 1, IndirectY),

        // CPX - compare x register
        // Op and flag results identical to CMP ops
        // N Z C
        Opcode::new(CPX, 0xE0, 2, 2, 0, Immediate),
        Opcode::new(CPX, 0xE4, 2, 3, 0, ZeroPage),
        Opcode::new(CPX, 0xEC, 3, 4, 0, Absolute),
        
        // CPY - compare y register
        // Op and flag results identical to CMP ops
        // N Z C
        Opcode::new(CPY, 0xC0, 2, 2, 0, Immediate),
        Opcode::new(CPY, 0xC4, 2, 3, 0, ZeroPage),
        Opcode::new(CPY, 0xCC, 3, 4, 0, Absolute),

        // DEC - decrement memory
        // N Z
        Opcode::new(DEC, 0xC6, 2, 5, 0, ZeroPage),
        Opcode::new(DEC, 0xD6, 2, 6, 0, ZeroPageX),
        Opcode::new(DEC, 0xCE, 3, 6, 0, Absolute),
        Opcode::new(DEC, 0xDE, 3, 7, 0, AbsoluteX),

        // EOR bitwise exclusive OR
        // N Z
        Opcode::new(EOR, 0x49, 2, 2, 0, Immediate),
        Opcode::new(EOR, 0x45, 2, 3, 0, ZeroPage),
        Opcode::new(EOR, 0x55, 2, 4, 0, ZeroPageX),
        Opcode::new(EOR, 0x4D, 3, 4, 0, Absolute),
        Opcode::new(EOR, 0x5D, 3, 4, 1, AbsoluteX),
        Opcode::new(EOR, 0x59, 3, 4, 1, AbsoluteY),
        Opcode::new(EOR, 0x41, 2, 6, 0, IndirectX),
        Opcode::new(EOR, 0x51, 2, 5, 1, IndirectY),
        
        // CLX, SEX - flag instructions 
        // Flags as noted
        Opcode::new(CLC, 0x18, 1, 2, 1, Implicit), // clear carry
        Opcode::new(SEC, 0x38, 1, 2, 1, Implicit), // set carry
        Opcode::new(CLI, 0x58, 1, 2, 1, Implicit), // clear interrupt
        Opcode::new(SEI, 0x78, 1, 2, 1, Implicit), // set interrupt
        Opcode::new(CLV, 0xB8, 1, 2, 1, Implicit), // clear overflow
        Opcode::new(CLD, 0xD8, 1, 2, 1, Implicit), // clear decimal
        Opcode::new(SED, 0xF8, 1, 2, 1, Implicit), // set decimal

        // INC - increment memory
        // N Z
        Opcode::new(INC, 0xE6, 2, 5, 0, ZeroPage),
        Opcode::new(INC, 0xF6, 2, 6, 0, ZeroPageX),
        Opcode::new(INC, 0xEE, 3, 6, 0, Absolute),
        Opcode::new(INC, 0xFE, 3, 7, 0, AbsoluteX),
        
        // JMP - jump
        // No flags
        Opcode::new(JMP, 0x4C, 3, 3, 0, Absolute),
        Opcode::new(JMP, 0x6C, 3, 3, 0, Indirect),

        // JSR - jump to subroutine
        // No flags
        Opcode::new(JSR, 0x20, 3, 6, 0, Absolute),

        // LDA - load accumulator
        // N Z
        Opcode::new(LDA, 0xA9, 2, 2, 0, Immediate),
        Opcode::new(LDA, 0xA5, 2, 3, 0, ZeroPage),
        Opcode::new(LDA, 0xB5, 2, 4, 0, ZeroPageX),
        Opcode::new(LDA, 0xAD, 3, 4, 0, Absolute),
        Opcode::new(LDA, 0xBD, 3, 4, 1, AbsoluteX),
        Opcode::new(LDA, 0xB9, 3, 4, 1, AbsoluteY),
        Opcode::new(LDA, 0xA1, 2, 6, 0, IndirectX),
        Opcode::new(LDA, 0xB1, 2, 5, 1, IndirectY),

        // LDX - load x register
        // N Z
        Opcode::new(LDX, 0xA2, 2, 2, 0, Immediate),
        Opcode::new(LDX, 0xA6, 2, 3, 0, ZeroPage),
        Opcode::new(LDX, 0xB6, 2, 4, 0, ZeroPageY),
        Opcode::new(LDX, 0xAE, 3, 4, 0, Absolute),
        Opcode::new(LDX, 0xBE, 3, 4, 1, AbsoluteY),

        // LDY - load y register
        // N Z
        Opcode::new(LDY, 0xA0, 2, 2, 0, Immediate),
        Opcode::new(LDY, 0xA4, 2, 3, 0, ZeroPage),
        Opcode::new(LDY, 0xB4, 2, 4, 0, ZeroPageX),
        Opcode::new(LDY, 0xAC, 3, 4, 0, Absolute),
        Opcode::new(LDY, 0xBC, 3, 4, 1, AbsoluteX),

        // LSR - logical shift right
        // N Z C
        Opcode::new(LSR, 0x4A, 1, 2, 0, Accumulator),
        Opcode::new(LSR, 0x46, 2, 5, 0, ZeroPage),
        Opcode::new(LSR, 0x56, 2, 6, 0, ZeroPageX),
        Opcode::new(LSR, 0x4E, 3, 6, 0, Absolute),
        Opcode::new(LSR, 0x5E, 3, 7, 0, AbsoluteX),

        // NOP - no operation
        // No Flags
        Opcode::new(NOP, 0xEA, 1, 2, 0, Implicit),
        
        // ORA - bitwise OR with accumulator
        // N Z
        Opcode::new(ORA, 0x09, 2, 2, 0, Immediate),
        Opcode::new(ORA, 0x05, 2, 3, 0, ZeroPage),
        Opcode::new(ORA, 0x15, 2, 4, 0, ZeroPageX),
        Opcode::new(ORA, 0x0D, 3, 4, 0, Absolute),
        Opcode::new(ORA, 0x1D, 3, 4, 1, AbsoluteX),
        Opcode::new(ORA, 0x19, 3, 4, 1, AbsoluteY),
        Opcode::new(ORA, 0x01, 2, 6, 0, IndirectX),
        Opcode::new(ORA, 0x11, 2, 5, 1, IndirectY),
        
        // Txx, DEx, INx - register instructions
        // N Z
        Opcode::new(TAX, 0xAA, 1, 2, 0, Implicit), // transfer a to x
        Opcode::new(TXA, 0x8A, 1, 2, 0, Implicit), // transfer x to a
        Opcode::new(DEX, 0xCA, 1, 2, 0, Implicit), // decrement x
        Opcode::new(INX, 0xE8, 1, 2, 0, Implicit), // increment x
        Opcode::new(TAY, 0xA8, 1, 2, 0, Implicit), // transfer a to y
        Opcode::new(TYA, 0x98, 1, 2, 0, Implicit), // transfer y to a
        Opcode::new(DEY, 0x88, 1, 2, 0, Implicit), // decrement y
        Opcode::new(INY, 0xC8, 1, 2, 0, Implicit), // increment y
        
        // ROL - rotate left
        // Carry into bit-0 and bit-7 into carry
        // N Z C
        Opcode::new(ROL, 0x2A, 1, 2, 0, Accumulator),
        Opcode::new(ROL, 0x26, 2, 5, 0, ZeroPage),
        Opcode::new(ROL, 0x36, 2, 6, 0, ZeroPageX),
        Opcode::new(ROL, 0x2E, 3, 6, 0, Absolute),
        Opcode::new(ROL, 0x3E, 3, 7, 0, AbsoluteX),

        // ROR - rotate right
        // Carry into bit-7 and bit-0 into carry
        // N Z C
        Opcode::new(ROR, 0x6A, 1, 2, 0, Accumulator),
        Opcode::new(ROR, 0x66, 2, 5, 0, ZeroPage),
        Opcode::new(ROR, 0x76, 2, 6, 0, ZeroPageX),
        Opcode::new(ROR, 0x6E, 3, 6, 0, Absolute),
        Opcode::new(ROR, 0x7E, 3, 7, 0, AbsoluteX),
        
        // RTI - return from interrupt
        // Retrives flags and pc from stack (in that order)
        // Return address is the actual address retrieved from the stack
        Opcode::new(RTI, 0x40, 1, 6, 0, Implicit),
        
        // RTS - return from subroutine
        // Retrives pc from stack (low-byte first)
        // Return address is the address retrieved from stack +1
        Opcode::new(RTS, 0x60, 1, 6, 0, Implicit),

        // SBC - subtract with carry
        // Results dependant on the decimal flag. In decimal mode subtraction
        // is carried out on the assumption that the values involved are
        // packed binary coded decimal
        // N V Z C
        Opcode::new(SBC, 0xE9, 2, 2, 0, Immediate),
        Opcode::new(SBC, 0xE5, 2, 3, 0, ZeroPage),
        Opcode::new(SBC, 0xF5, 2, 3, 0, ZeroPageX),
        Opcode::new(SBC, 0xED, 3, 4, 0, Absolute),
        Opcode::new(SBC, 0xFD, 3, 4, 1, AbsoluteX),
        Opcode::new(SBC, 0xF9, 3, 4, 1, AbsoluteY),
        Opcode::new(SBC, 0xE1, 2, 6, 0, IndirectX),
        Opcode::new(SBC, 0xF1, 2, 5, 1, IndirectY),

        // STA - store accumulator
        // No flags
        Opcode::new(STA, 0x85, 2, 3, 0, ZeroPage),
        Opcode::new(STA, 0x95, 2, 4, 0, ZeroPageX),
        Opcode::new(STA, 0x8D, 3, 4, 0, Absolute),
        Opcode::new(STA, 0x9D, 3, 5, 0, AbsoluteX),
        Opcode::new(STA, 0x99, 3, 5, 0, AbsoluteY),
        Opcode::new(STA, 0x81, 2, 6, 0, IndirectX),
        Opcode::new(STA, 0x91, 2, 6, 0, IndirectY),

        // Pxx - stack instructions
        // No flags
        Opcode::new(TXS, 0x9A, 2, 2, 0, Implicit), // transfer x to stack ptr
        Opcode::new(TSX, 0xBA, 2, 2, 0, Implicit), // transfer stack ptr to x
        Opcode::new(PHA, 0x48, 2, 3, 0, Implicit), // push accumulator
        Opcode::new(PLA, 0x68, 2, 4, 0, Implicit), // pull accumulator
        Opcode::new(PHP, 0x08, 2, 3, 0, Implicit), // push processor status
        Opcode::new(PLP, 0x28, 2, 4, 0, Implicit), // pull processor status
        
        // STX - store x register
        // No flags
        Opcode::new(STX, 0x86, 2, 3, 0, ZeroPage),
        Opcode::new(STX, 0x96, 2, 4, 0, ZeroPageY),
        Opcode::new(STX, 0x8E, 3, 4, 0, Absolute),

        // STR - store y register
        // No flags
        Opcode::new(STY, 0x84, 2, 3, 0, ZeroPage),
        Opcode::new(STY, 0x94, 2, 4, 0, ZeroPageY),
        Opcode::new(STY, 0x8C, 3, 4, 0, Absolute),
    ];
    
    pub static ref CPU_OPCODES_MAP: HashMap<u8, &'static Opcode> =
        NMOS_6502_OPCODES.iter()
            .map(|op| (op.code, op))
            .collect();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_duplicate_nmos_6502_ops() {
        let ops = NMOS_6502_OPCODES.iter()
            .map(|op| op.code)
            .collect::<Vec<_>>();
        
        let mut sorted = ops.clone();
        sorted.sort();
        sorted.dedup();
        assert_eq!(sorted.len(), ops.len());
    }
}