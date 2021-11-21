use nom::IResult;
use nom::branch::alt;
use nom::bytes::complete::{is_a, tag_no_case};
use nom::character::is_hex_digit;
use nom::combinator::{not, peek};
use nom::error::{ErrorKind, ParseError, VerboseError, context, make_error};
use nom::bytes::complete::take;
use nom::sequence::{pair, preceded, terminated};
use nom::character::complete::{digit1, hex_digit1, line_ending, not_line_ending, one_of, space0};
use nom::multi::{count, many0};

use crate::cpu::Mnemonic;
use crate::cpu::addr::AddressMode;
use crate::cpu::prog::instructions::Operand;


struct OperandMode {
    operand: Operand,
    mode: AddressMode,
}

/// Combinator to take the first three chars and parse a Mnemonic
fn mnemonic(s: &str) -> IResult<&str, Mnemonic> {
    let (rem, res) = take(3_usize)(s)?;
    match res.to_uppercase().parse() {
        Ok(mnem) => Ok((rem, mnem)),
        Err(_) => Err(nom::Err::Error(make_error(s, ErrorKind::Tag)))
    }
}

/// Combinator to read to the next line ending
fn line(s: &str) -> IResult<&str, &str> {
    preceded(
        space0,
        terminated(
            not_line_ending, // replace this bit with the goods and return an instruction
            line_ending,
        ),
    )(s)
}

/// Base parser combinator to read a whole assembly program
fn program(s: &str) -> IResult<&str, Vec<&str>> {
    many0(
        line,
    )(s)
}

// ///
// fn mode(s: &str) -> IResult<&str, Operand> {
//     todo!()
// }

fn operand(s: &str)-> IResult<&str, OperandMode> {
    todo!()
}

fn implicit(s: &str) -> IResult<&str, OperandMode> {
    not(one_of("aA#$("))(s)
        .map(|(rem,_res)| (rem, OperandMode { operand: Operand::None, mode: AddressMode::Implicit }))
}

fn accumulator(s: &str) -> IResult<&str, OperandMode> {
    tag_no_case("A")(s)
        .map(|(rem, _res)| (rem, OperandMode { operand: Operand::None, mode: AddressMode::Accumulator }))
}

fn hex_byte(s: &str) -> IResult<&str, u8> {
    let (rem, res) = hex_digit1(s)?;
    match res.len() {
        2 => Ok((rem, u8::from_str_radix(res, 16).unwrap())),
        _ => Err(nom::Err::Error(make_error(s, ErrorKind::HexDigit))),
    }
}

fn hex_double_byte(s: &str) -> IResult<&str, u16> {
    let (rem, res) = hex_digit1(s)?;
    match res.len() {
        4 => Ok((rem, u16::from_str_radix(res, 16).unwrap())),
        _ => Err(nom::Err::Error(make_error(s, ErrorKind::HexDigit))),
    }
}


// fn hex_byte(s: &str) -> IResult<&str, u8> {
//     let (rem, res) = count(is_hex_digit,2)(s)?;
//     match peek(not(hex_digit1))(rem) {
//         Ok(mnem) => Ok((rem, mnem)),
//         Err(_) => Err(nom::Err::Error(make_error(s, ErrorKind::Tag)))
//     }
// }

fn immediate(s: &str) -> IResult<&str, OperandMode> {
    preceded(
        tag_no_case("#"),
        hex_byte,
    )(s)
        .map(|(rem, res)| {
            (
                rem,
                OperandMode {
                    operand: Operand::Word(res),
                    mode: AddressMode::Immediate,
                }   
            )
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
            (
                rem,
                OperandMode {
                    operand: Operand::Word(res),
                    mode: AddressMode::ZeroPage,
                }   
            )
        })?;
    
    // if anything follows then this is actually another mode
    match rem.chars().nth(0) {
        Some(',') | Some(')') => Err(nom::Err::Error(make_error(s, ErrorKind::Fail))),
        _ => Ok((rem, res)),
    }
}

fn zeropage_x(s: &str) -> IResult<&str, OperandMode> {
    terminated(
        zp_addr,
        tag_no_case(",X"),
    )(s)
        .map(|(rem, res)| {
            (
                rem,
                OperandMode {
                    operand: Operand::Word(res),
                    mode: AddressMode::ZeroPageX,
                }
            )
        })
}

fn zeropage_y(s: &str) -> IResult<&str, OperandMode> {
    terminated(
        zp_addr,
        tag_no_case(",Y"),
    )(s)
        .map(|(rem, res)| {
            (
                rem,
                OperandMode {
                    operand: Operand::Word(res),
                    mode: AddressMode::ZeroPageY,
                }
            )
        })
}

fn relative(s: &str) -> IResult<&str, OperandMode> {
    preceded(
        tag_no_case("*"),
        pair(one_of("+-"), digit1),
    )(s)
        .map(|(rem,res)| {
            (
                rem, 
                OperandMode {
                    operand: Operand::Word(i8::from_str_radix(&(res.0.to_string() + res.1), 10).unwrap() as u8),
                    mode: AddressMode::Relative
                }
            )
        })
}

fn absolute(s: &str) -> IResult<&str, OperandMode> {
    let (rem, res) = abs_addr(s)
        .map(|(rem, res)| {
            (
                rem,
                OperandMode {
                    operand: Operand::DoubleWord(res),
                    mode: AddressMode::Absolute,
                }   
            )
        })?;
    
    // if anything follows then this is actually another mode
    match rem.chars().nth(0) {
        Some(',') | Some(')') => Err(nom::Err::Error(make_error(s, ErrorKind::TooLarge))),
        _ => Ok((rem, res)),
    }
}

fn absolute_x(s: &str) -> IResult<&str, OperandMode> {
    terminated(
        abs_addr,
        tag_no_case(",X"),
    )(s)
        .map(|(rem, res)| {
            (
                rem,
                OperandMode {
                    operand: Operand::DoubleWord(res),
                    mode: AddressMode::AbsoluteX
                }
            )
        })
}

fn absolute_y(s: &str) -> IResult<&str, OperandMode> {
    terminated(
        abs_addr,
        tag_no_case(",Y"),
    )(s)
        .map(|(rem, res)| {
            (
                rem,
                OperandMode {
                    operand: Operand::DoubleWord(res),
                    mode: AddressMode::AbsoluteY
                }
            )
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
            (
                rem,
                OperandMode {
                    operand: Operand::Word(res),
                    mode: AddressMode::Indirect,
                }
            )
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
            (
                rem,
                OperandMode {
                    operand: Operand::Word(res),
                    mode: AddressMode::IndirectX,
                }
            )
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
            (
                rem,
                OperandMode {
                    operand: Operand::Word(res),
                    mode: AddressMode::IndirectX,
                }
            )
        })
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_mnemonic() {
        assert_eq!(mnemonic("LDA"), Ok(("", Mnemonic::LDA)));
        assert_eq!(mnemonic("TAXABC"), Ok(("ABC", Mnemonic::TAX)));
    }

    #[test]
    fn test_parse_line() {
        assert_eq!(line("asdfg\nxcvb"), Ok(("xcvb", "asdfg")));
    }

    #[test]
    fn test_parse_program() {
        assert_eq!(program("LDA #02\nADC #03\nBRK\n"), Ok(("", vec!["LDA #02", "ADC #03", "BRK"])));
    }
}