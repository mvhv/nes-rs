use nom::{IResult, bytes::complete::{tag, take_while_m_n}, combinator::map_res, sequence::tuple};

use ::crate::cpu::code::instructions::{Instruction, Operand};
use ::crate::cpu::ops::Mnemonic;


fn parse_assembly(input: &str) -> IResult {
    
}