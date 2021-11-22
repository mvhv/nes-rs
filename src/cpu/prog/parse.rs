use nom::IResult;
use nom::branch::alt;
use nom::bytes::complete::{tag_no_case, take};
use nom::combinator::{eof, opt, peek, success};
// use nom::combinator::{not, peek};
use nom::error::{ErrorKind, make_error};
use nom::sequence::{pair, preceded, terminated};
use nom::character::complete::{digit1, hex_digit1, line_ending, not_line_ending, one_of, space0, space1};
use nom::multi::{many0, many_till};

use nom::Err as NomErr; // typedef to make error handling less confusing

use crate::cpu::Mnemonic;
use crate::cpu::AddressMode::{self, *};
use crate::cpu::prog::instructions::Operand::{self, *};

use super::instructions::Instruction;


#[derive(Debug, PartialEq, Eq)]
struct OperandMode {
    operand: Operand,
    mode: AddressMode,
}

impl OperandMode {
    pub fn new(operand: Operand, mode: AddressMode) -> Self {
        Self { operand, mode }
    }
}

/// Combinator to take the first three chars and parse a Mnemonic
fn mnemonic(s: &str) -> IResult<&str, Mnemonic> {
    // better to change this to gobble all alphas
    // else labels like "LDATHING:" will be parsed as menms
    let (rem, res) = take(3_usize)(s)?; 
    match res.to_uppercase().parse() {
        Ok(mnem) => Ok((rem, mnem)),
        Err(_) => Err(NomErr::Error(make_error(s, ErrorKind::Tag)))
    }
}

/// Combinator to read to the next line ending
fn line(s: &str) -> IResult<&str, Option<Instruction>> {
    preceded(
        space0,
        terminated(
            opt(instruction),
            pair(
                opt(
                    pair(
                        space0,
                        comment
                    ),
                ),
                alt((
                    line_ending,
                    eof,
                ))
            ),
        ),
    )(s)
}

/// Base parser combinator to read a whole assembly program
fn program(s: &str) -> IResult<&str, Vec<Instruction>> {
    many_till(
        line,
        eof,
    )(s)
        .map(|(rem, (res, end))| {
            (rem, res.into_iter().filter_map(|x| x).collect())
        })
}

// ///
// fn mode(s: &str) -> IResult<&str, Operand> {
//     todo!()
// }

fn operand(s: &str)-> IResult<&str, OperandMode> {
    alt((
        accumulator,
        immediate,
        zeropage,
        zeropage_x,
        zeropage_y,
        relative,
        absolute,
        absolute_x,
        absolute_y,
        indirect,
        indirect_x,
        indirect_y,
        implicit, // implicit as the fall through
    ))(s)
}

/// Parser combinator for implicit operands.
/// Always succeeds, so only to be used as the fallthrough case.
fn implicit(s: &str) -> IResult<&str, OperandMode> {
    success("implicit")(s)
        .map(|(rem,_res)| (rem, OperandMode::new(None, Implicit)))
}

fn accumulator(s: &str) -> IResult<&str, OperandMode> {
    tag_no_case("A")(s)
        .map(|(rem, _res)| (rem, OperandMode::new(None, Accumulator)))
}

fn hex_byte(s: &str) -> IResult<&str, u8> {
    let (rem, res) = hex_digit1(s)?;
    match res.len() {
        2 => Ok((rem, u8::from_str_radix(res, 16).unwrap())),
        _ => Err(NomErr::Error(make_error(s, ErrorKind::HexDigit))),
    }
}

fn hex_double_byte(s: &str) -> IResult<&str, u16> {
    let (rem, res) = hex_digit1(s)?;
    match res.len() {
        4 => Ok((rem, u16::from_str_radix(res, 16).unwrap())),
        _ => Err(NomErr::Error(make_error(s, ErrorKind::HexDigit))),
    }
}

fn immediate(s: &str) -> IResult<&str, OperandMode> {
    preceded(
        tag_no_case("#"),
        hex_byte,
    )(s)
        .map(|(rem, res)| {
            (rem, OperandMode::new(Word(res), Immediate))
        })
}

fn zp_addr(s: &str) -> IResult<&str, u8> {
    preceded(
        tag_no_case("$"),
        hex_byte,
    )(s)
}

fn abs_addr(s: &str) -> IResult<&str, u16> {
    preceded(
        tag_no_case("$"),
        hex_double_byte,
    )(s)
}

fn zeropage(s: &str) -> IResult<&str, OperandMode> {
    let (rem, res) = zp_addr(s)
        .map(|(rem, res)| {
            (rem, OperandMode::new(Word(res), ZeroPage))
        })?;
    
    // if anything follows then this is actually another mode
    match rem.chars().nth(0) {
        Some(',') | Some(')') => Err(NomErr::Error(make_error(s, ErrorKind::Fail))),
        _ => Ok((rem, res)),
    }
}

fn zeropage_x(s: &str) -> IResult<&str, OperandMode> {
    terminated(
        zp_addr,
        tag_no_case(",X"),
    )(s)
        .map(|(rem, res)| {
            (rem, OperandMode::new(Word(res), ZeroPageX))
        })
}

fn zeropage_y(s: &str) -> IResult<&str, OperandMode> {
    terminated(
        zp_addr,
        tag_no_case(",Y"),
    )(s)
        .map(|(rem, res)| {
            (rem, OperandMode::new(Word(res), ZeroPageY))
        })
}

fn relative(s: &str) -> IResult<&str, OperandMode> {
    preceded(
        tag_no_case("*"),
        pair(one_of("+-"), digit1),
    )(s)
        .map(|(rem,res)| {
            (rem, OperandMode::new(
                Word(i8::from_str_radix(&(res.0.to_string() + res.1), 10).unwrap() as u8),
                Relative
            ))
        })
}

fn absolute(s: &str) -> IResult<&str, OperandMode> {
    let (rem, res) = abs_addr(s)
        .map(|(rem, res)| {
            (rem, OperandMode::new(DoubleWord(res), Absolute))
        })?;
    
    // if anything follows then this is actually another mode
    match rem.chars().nth(0) {
        Some(',') | Some(')') => Err(NomErr::Error(make_error(s, ErrorKind::TooLarge))),
        _ => Ok((rem, res)),
    }
}

fn absolute_x(s: &str) -> IResult<&str, OperandMode> {
    terminated(
        abs_addr,
        tag_no_case(",X"),
    )(s)
        .map(|(rem, res)| {
            (rem, OperandMode::new(DoubleWord(res), AbsoluteX))
        })
}

fn absolute_y(s: &str) -> IResult<&str, OperandMode> {
    terminated(
        abs_addr,
        tag_no_case(",Y"),
    )(s)
        .map(|(rem, res)| {
            (rem, OperandMode::new(DoubleWord(res), AbsoluteY))
        })
}

fn indirect(s: &str) -> IResult<&str, OperandMode> {
    preceded(
        tag_no_case("("),
        terminated(
            zp_addr,
            tag_no_case(")"),
        ),
    )(s)
        .map(|(rem, res)| {
            (rem, OperandMode::new(Word(res), Indirect))
        })
}

fn indirect_x(s: &str) -> IResult<&str, OperandMode> {
    preceded(
        tag_no_case("("),
        terminated(
            zp_addr,
            tag_no_case(",X)"),
        ),
    )(s)
        .map(|(rem, res)| {
            (rem, OperandMode::new(Word(res), IndirectX))
        })
}

fn indirect_y(s: &str) -> IResult<&str, OperandMode> {
    preceded(
        tag_no_case("("),
        terminated(
            zp_addr,
            tag_no_case("),Y"),
        ),
    )(s)
        .map(|(rem, res)| {
            (rem, OperandMode::new(Word(res), IndirectY))
        })
}

fn instruction(s: &str) -> IResult<&str, Instruction> {
    pair(
        mnemonic,
        opt(
            pair(
                space1,
                operand
            )
        )
    )(s)
        .map(|(rem, res)| {
            match res {
                (mnem, Some((_, OperandMode { operand, mode}))) => (rem, Instruction::new(mnem, operand, mode)),
                (mnem, Option::None) => (rem, Instruction::new(mnem, Operand::None, AddressMode::Implicit))
            }
            
        })
}

fn comment(s: &str) -> IResult<&str, &str> {
    preceded(
        tag_no_case(";"),
        not_line_ending,
    )(s)
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::cpu::AddressMode::{self, *};
    use crate::cpu::prog::instructions::Operand::{self, *};

    #[test]
    fn test_parse_zp_addr() {
        assert_eq!(
            zp_addr("$23"),
            Ok(("", 0x23))
        );

        assert_eq!(
            zp_addr("$FF Hello"),
            Ok((" Hello", 0xFF))
        );

        assert_eq!(
            zp_addr("$00,World"),
            Ok((",World", 0x00))
        );

        assert_eq!(
            zp_addr("$003)Bye"),
            Err(NomErr::Error(make_error("003)Bye", ErrorKind::HexDigit))) // this really should keep the $
        );
    }

    #[test]
    fn test_parse_accumulator() {
        assert_eq!(
            accumulator("A ;a comment\nLDA #02\n"),
            Ok((" ;a comment\nLDA #02\n", OperandMode::new(None, Accumulator))),
        );
    }

    #[test]
    fn test_parse_implicit() {
        assert_eq!(
            implicit(""),
            Ok(("", OperandMode::new(None, Implicit)))
        );
    }

    #[test]
    fn test_parse_relative() {
        assert_eq!(
            relative("*-23"),
            Ok(("", OperandMode::new(Word(-23_i8 as u8), Relative)))
        );
    }

    #[test]
    fn test_parse_operand() {
        assert_eq!(
            operand("$34,Y ;comment\nLDA"),
            Ok((" ;comment\nLDA", OperandMode::new(Word(0x34), ZeroPageY)))
        );

        assert_eq!(
            operand("($34,X) ;comment\nLDA"),
            Ok((" ;comment\nLDA", OperandMode::new(Word(0x34), IndirectX)))
        );

        assert_eq!(
            operand("$3423,X ;comment\nLDA"),
            Ok((" ;comment\nLDA", OperandMode::new(DoubleWord(0x3423), AbsoluteX)))
        );

        assert_eq!(
            operand("*-23 ;comment\nLDA"),
            Ok((" ;comment\nLDA", OperandMode::new(Word(-23i8 as u8), Relative)))
        );
    }

    #[test]
    fn test_parse_mnemonic() {
        assert_eq!(mnemonic("LDA"), Ok(("", Mnemonic::LDA)));
        assert_eq!(mnemonic("TAXABC"), Ok(("ABC", Mnemonic::TAX)));
    }

    #[test]
    fn test_parse_instruction() {
        assert_eq!(
            instruction("LDA #02\n"),
            Ok((
                "\n",
                Instruction::new(
                    Mnemonic::LDA,
                    Operand::Word(0x02),
                    AddressMode::Immediate,
                )
            ))
        );

        assert_eq!(
            instruction("RTS\n"),
            Ok((
                "\n",
                Instruction::new(
                    Mnemonic::RTS,
                    Operand::None,
                    AddressMode::Implicit,
                )
            ))
        );

        assert_eq!(
            instruction("BPL *-23\n"),
            Ok((
                "\n",
                Instruction::new(
                    Mnemonic::BPL,
                    Operand::Word(-23_i8 as u8),
                    AddressMode::Relative,
                )
            ))
        );
        
        assert_eq!(
            instruction("DEC $FF23,X\n"),
            Ok((
                "\n",
                Instruction::new(
                    Mnemonic::DEC,
                    Operand::DoubleWord(0xFF23),
                    AddressMode::AbsoluteX,
                )
            ))
        );
    }

    #[test]
    fn test_parse_comment() {
        assert_eq!(
            comment(";a comment\nLDA #02\n"),
            Ok(("\nLDA #02\n", "a comment")),
        );
    }

    #[test]
    fn test_parse_line() {
        assert_eq!(
            line("LDA #02\nTAX\nBRK\n"),
            Ok((
                "TAX\nBRK\n",
                Some(Instruction::new(
                    Mnemonic::LDA,
                    Operand::Word(0x02),
                    AddressMode::Immediate,
                )),
            ))
        );
    }

    #[test]
    fn test_parse_program() {
        assert_eq!(
            program("LDA #02\nDEC $FF23,X ;Nonsense comment\n;No content\nBRK\n"),
            Ok((
                "",
                vec![
                    Instruction::new(
                        Mnemonic::LDA,
                        Operand::Word(0x02),
                        AddressMode::Immediate,
                    ),
                    Instruction::new(
                        Mnemonic::DEC,
                        Operand::DoubleWord(0xFF23),
                        AddressMode::AbsoluteX,
                    ),
                    Instruction::new(
                        Mnemonic::BRK,
                        Operand::None,
                        AddressMode::Implicit,
                    )
                ]
            ))
        );

        let code_chunk =
        r#"lda #10 ;put the hex number $10 (dec 16) in register A
        sta $12  ;store value of register A at address hex $12
        lda #0f ;put the hex number $0f (dec 15) in register A
        sta $14  ;store value of register A at address hex $14
      
        ;the most significant bytes are all set to hex $04
        ;which is the third 8x32 strip.
        lda #04 ;put the hex number $04 in register A
        sta $11  ;store value of register A at address hex 11
        sta $13  ;store value of register A at address hex 13
        sta $15  ;store value of register A at address hex 15
        rts      ;return
        "#;

        program(code_chunk).unwrap();
        // assert_eq!(
        //     program(code_chunk),
        //     Ok((
        //         "",
        //         vec![]
        //     ))
        // );
    }
}